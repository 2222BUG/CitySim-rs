//! 寻路系统：确定性 A* 与机制 B 预算让步（主文档 2.2 架构分层；ADR-NEW-24 决策 2/3）。
//! 权威 crate。
//!
//! Stage 2A 实装（2026-11-19）：
//! - A* 唯一性五条（ADR-NEW-24 决策 2）：启发式=曼哈顿定点；open 集排序键 (f, h, node_id)
//!   全序升序；邻居展开=CSR 数组序；closed 首展闭合、g 严格更小才 reopen；禁 BinaryHeap。
//! - 机制 B（ADR-NEW-20 三硬约束）：B_k = min(b·g^k, L)，g=2，L=|V|；单 tick 原子搜索
//!   （方案③：跨 tick 不保存部分搜索状态，让步即丢弃本 tick 进度、下 tick 以更大预算重做）；
//!   解析上界 k ≤ ⌈log_g(|V|/b)⌉ + 1 tick 内完成或进入受阻判定。
//! - 双计数器分离（ADR-NEW-20）：budget_yield_count（预算让步）与 stale_recompute_count
//!   （过期重算）互不共用上限。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use sim_network::Network;

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "寻路系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

// ---------------- 机制 B 预算策略 ----------------

/// 机制 B 参数（ADR-NEW-24 决策 3）：b=每 tick 节点预算，g=放大幅（默认 2），L=|V|。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetPolicy {
    pub b: u64,
    pub g: u64,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        BudgetPolicy { b: 64, g: 2 }
    }
}

impl BudgetPolicy {
    /// B_k = min(b·g^k, L)（k=连续让步次数）。
    pub fn budget_for_yield(&self, yields: u64, n_nodes: u64) -> u64 {
        let l = n_nodes.max(1);
        let mut acc = self.b.max(1);
        for _ in 0..yields {
            acc = acc.saturating_mul(self.g.max(2));
            if acc >= l {
                break;
            }
        }
        acc.min(l)
    }

    /// 解析上界：k ≤ ⌈log_g(L/b)⌉ + 1 个 tick 内完成或受阻（ADR-NEW-20 三硬约束）。
    pub fn max_ticks_to_terminal(&self, n_nodes: u64) -> u64 {
        let l = n_nodes.max(1);
        let b = self.b.max(1).min(l);
        let mut k = 0u64;
        let mut acc = b;
        while acc < l {
            acc = acc.saturating_mul(self.g.max(2));
            k += 1;
        }
        k + 1
    }
}

// ---------------- 搜索结果 ----------------

/// 单次（单 tick 原子）搜索结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FindOutcome {
    /// 找到路径：边 ID 序列 + 总代价 + 本次展开节点数。
    Complete { path: Vec<u64>, cost: i64, expanded: u64 },
    /// 本 tick 节点预算耗尽（让步；不保存中间状态，下 tick 以 B_k 重做）。
    Yielded { expanded: u64 },
    /// open 集耗尽未达目标：不可达（进入受阻终态的判定输入）。
    Unreachable,
}

/// 寻路器状态：双计数器 + 机制 B 执行（ADR-NEW-20）。
#[derive(Clone, Debug, Default)]
pub struct PathFinder {
    pub policy: BudgetPolicy,
    /// 预算让步计数（连续让步，成功/受阻即清零）。
    pub budget_yield_count: u64,
    /// 过期重算计数（与让步分离，独立上限由调用方约束）。
    pub stale_recompute_count: u64,
}

impl PathFinder {
    pub fn new(policy: BudgetPolicy) -> Self {
        PathFinder { policy, ..Default::default() }
    }

    /// 发起一次寻路请求（单 tick 原子；机制 B 预算 B_k）。
    pub fn request(&mut self, net: &Network, coords: &[(i64, i64)], start: u64, goal: u64) -> FindOutcome {
        let budget = self.policy.budget_for_yield(self.budget_yield_count, net.node_count());
        let out = find_once(net, coords, start, goal, budget);
        match &out {
            FindOutcome::Complete { .. } | FindOutcome::Unreachable => {
                self.budget_yield_count = 0;
            }
            FindOutcome::Yielded { .. } => {
                self.budget_yield_count = self.budget_yield_count.saturating_add(1);
            }
        }
        out
    }

    /// 全局每 tick 配额下的寻路请求（债务 #29 定案，ADR-NEW-24 决策 3 补充）。
    ///
    /// B_k ≤ min(b·2^k, L, budget.remaining)；实耗展开数扣减 `budget.remaining`；
    /// 配额耗尽即让步（`Yielded { expanded: 0 }`，不消耗预算）。
    /// 多请求处理顺序（请求 ID 稳定序）与每 tick `TickBudget::reset()` 由调用方负责。
    pub fn request_with_budget(
        &mut self,
        net: &Network,
        coords: &[(i64, i64)],
        start: u64,
        goal: u64,
        budget: &mut TickBudget,
    ) -> FindOutcome {
        if budget.remaining == 0 {
            self.budget_yield_count = self.budget_yield_count.saturating_add(1);
            return FindOutcome::Yielded { expanded: 0 };
        }
        let b_k = self
            .policy
            .budget_for_yield(self.budget_yield_count, net.node_count())
            .min(budget.remaining);
        let out = find_once(net, coords, start, goal, b_k);
        if let FindOutcome::Complete { expanded, .. } | FindOutcome::Yielded { expanded } = &out {
            budget.remaining = budget.remaining.saturating_sub(*expanded);
        }
        match &out {
            FindOutcome::Complete { .. } | FindOutcome::Unreachable => {
                self.budget_yield_count = 0;
            }
            FindOutcome::Yielded { .. } => {
                self.budget_yield_count = self.budget_yield_count.saturating_add(1);
            }
        }
        out
    }

    /// 过期重算记账（与让步计数分离；ADR-NEW-20 双计数器）。
    pub fn note_stale_recompute(&mut self) {
        self.stale_recompute_count = self.stale_recompute_count.saturating_add(1);
    }
}

/// 全局每 tick 寻路配额（债务 #29 定案）。
///
/// `remaining` = 本 tick 尚可消耗的节点展开数（确定性工作量单位，对齐 ADR-NEW-14 条 3）；
/// 每 tick 由调用方在 tick 边界 `reset()` 归满（每 tick 预算独立）。
/// 多请求共享：请求实耗扣减，耗尽后其余请求让步——全局每 tick 总展开 ≤ `quota`。
#[derive(Clone, Debug)]
pub struct TickBudget {
    pub quota: u64,
    pub remaining: u64,
}

impl TickBudget {
    pub fn new(quota: u64) -> Self {
        TickBudget { quota, remaining: quota }
    }

    /// tick 边界重置。
    pub fn reset(&mut self) {
        self.remaining = self.quota;
    }
}

// ---------------- 确定性 A*（ADR-NEW-24 决策 2） ----------------

fn h_manhattan(coords: &[(i64, i64)], node: u64, goal: u64) -> i64 {
    let (x1, y1) = coords.get(node as usize).copied().unwrap_or((0, 0));
    let (x2, y2) = coords.get(goal as usize).copied().unwrap_or((0, 0));
    // 溢出契约（ADR-NEW-21 决策 3）：禁止静默环绕，i128 中间值 + 饱和收敛到 i64。
    let dx = (x1 as i128 - x2 as i128).abs();
    let dy = (y1 as i128 - y2 as i128).abs();
    (dx + dy).min(i64::MAX as i128) as i64
}

/// 单 tick 原子搜索：展开节点数不超过 node_budget；确定性五条内联。
///
/// 前提（ADR-NEW-24）：边代价下限 ≥ 边几何曼哈顿长度，h 可采纳。
pub fn find_once(
    net: &Network,
    coords: &[(i64, i64)],
    start: u64,
    goal: u64,
    node_budget: u64,
) -> FindOutcome {
    if start == goal {
        return FindOutcome::Complete { path: Vec::new(), cost: 0, expanded: 0 };
    }
    // open 集：BTreeMap<(f, h, node_id), ()> 升序弹出 = 排序键全序（决策 2 第 2 条）
    let h0 = h_manhattan(coords, start, goal);
    let mut open: BTreeMap<(i64, i64, u64), ()> = BTreeMap::new();
    open.insert((h0, h0, start), ());
    let mut g: BTreeMap<u64, i64> = BTreeMap::new();
    g.insert(start, 0);
    // parent 记录 (父节点, 松弛时实际使用的边 ID)：平行同向边存在时路径边集与代价严格一致。
    let mut parent: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
    let mut closed: BTreeSet<u64> = BTreeSet::new();
    let mut expanded = 0u64;

    while let Some(((_f, _h, node), ())) = open.pop_first() {
        if node == goal {
            // 回溯路径（使用松弛时记录的边 ID）
            let mut path = Vec::new();
            let mut cur = goal;
            while let Some(&(p, eid)) = parent.get(&cur) {
                path.push(eid);
                cur = p;
            }
            path.reverse();
            return FindOutcome::Complete { path, cost: g[&goal], expanded };
        }
        if expanded >= node_budget {
            // 预算耗尽：让步（不保存中间状态，方案③）
            return FindOutcome::Yielded { expanded };
        }
        if !closed.insert(node) {
            continue; // 已闭合且未被更优 g reopen
        }
        expanded += 1;
        let g_cur = g[&node];
        // 邻居展开序 = CSR 数组序（决策 2 第 3 条）
        for (eid, to, cost) in net.neighbors(node) {
            // 溢出契约（ADR-NEW-21 决策 3）：饱和运算，禁止静默环绕。
            let new_g = g_cur.saturating_add(cost);
            let known = g.get(&to).copied();
            if known.is_some_and(|k| new_g >= k) {
                continue; // 仅严格更小才松弛/reopen（决策 2 第 4 条）
            }
            g.insert(to, new_g);
            parent.insert(to, (node, eid));
            closed.remove(&to); // 允许 reopen
            let hh = h_manhattan(coords, to, goal);
            let ff = new_g.saturating_add(hh);
            open.insert((ff, hh, to), ());
        }
    }
    FindOutcome::Unreachable
}

// ---------------- 测试 ----------------

#[cfg(test)]
mod tests {
    use super::*;
    use sim_network::Edge;
    use sim_network::{Op, Tx};

    fn grid(w: u64, h: u64) -> (Network, Vec<(i64, i64)>) {
        // w×h 网格（mm 单位 1000/格），边代价=曼哈顿长度（满足可采纳前提）
        let mut net = Network::new();
        let mut coords = Vec::new();
        for y in 0..h {
            for x in 0..w {
                coords.push((x as i64 * 1000, y as i64 * 1000));
            }
        }
        let mut tid = 1u64;
        let mut eid = 1u64;
        let node = |x: u64, y: u64| y * w + x;
        for y in 0..h {
            for x in 0..w {
                if x + 1 < w {
                    net.publish(Tx {
                        id: tid,
                        ops: vec![Op::AddEdge(sim_network::Edge {
                            id: eid,
                            a: node(x, y),
                            b: node(x + 1, y),
                            cost: 1000,
                            geom: vec![(x as i64 * 1000, y as i64 * 1000), ((x + 1) as i64 * 1000, y as i64 * 1000)],
                        })],
                    })
                    .unwrap();
                    tid += 1;
                    eid += 1;
                }
                if y + 1 < h {
                    net.publish(Tx {
                        id: tid,
                        ops: vec![Op::AddEdge(sim_network::Edge {
                            id: eid,
                            a: node(x, y),
                            b: node(x, y + 1),
                            cost: 1000,
                            geom: vec![(x as i64 * 1000, y as i64 * 1000), (x as i64 * 1000, (y + 1) as i64 * 1000)],
                        })],
                    })
                    .unwrap();
                    tid += 1;
                    eid += 1;
                }
            }
        }
        (net, coords)
    }

    /// 参考实现：Dijkstra（仅测试用，验证最优性对拍）。
    fn dijkstra(net: &Network, start: u64, goal: u64) -> Option<i64> {
        let mut dist: BTreeMap<u64, i64> = BTreeMap::new();
        dist.insert(start, 0);
        let mut done: BTreeSet<u64> = BTreeSet::new();
        loop {
            let next = dist
                .iter()
                .filter(|(n, _)| !done.contains(n))
                .min_by_key(|(n, d)| (**d, **n))
                .map(|(n, d)| (*n, *d));
            let (u, du) = next?;
            if u == goal {
                return Some(du);
            }
            done.insert(u);
            for (_, to, cost) in net.neighbors(u) {
                let nd = du.saturating_add(cost);
                if dist.get(&to).is_none_or(|&d| nd < d) {
                    dist.insert(to, nd);
                }
            }
        }
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
        assert!(!LAYER.is_empty());
    }

    /// 最优性对拍（ADR-NEW-24 退出判据）：A* 代价 == Dijkstra 代价。
    #[test]
    fn astar_matches_dijkstra() {
        let (net, coords) = grid(5, 5);
        for goal in [4u64, 12, 20, 24] {
            let out = find_once(&net, &coords, 0, goal, 10_000);
            let FindOutcome::Complete { cost, .. } = out else { panic!("must complete") };
            assert_eq!(Some(cost), dijkstra(&net, 0, goal), "goal={goal}");
        }
    }

    /// 唯一性（ADR-NEW-24 退出判据）：同输入重复运行路径逐边一致。
    #[test]
    fn astar_is_unique() {
        let (net, coords) = grid(6, 6);
        let a = find_once(&net, &coords, 0, 35, 10_000);
        let b = find_once(&net, &coords, 0, 35, 10_000);
        assert_eq!(a, b);
    }

    /// 平局打破确定性（决策 2 第 2 条）：两条等价路径按 (f,h,node_id) 选 node_id 小者。
    /// 2×2 网格：0->1->3 与 0->2->3 等价，node 1 < node 2 ⇒ 走 0->1->3。
    #[test]
    fn tie_break_is_deterministic() {
        let (net, coords) = grid(2, 2);
        let FindOutcome::Complete { path, .. } = find_once(&net, &coords, 0, 3, 100) else {
            panic!("must complete")
        };
        // 边 ID：0->1 是水平第一条（id=1），1->3 是纵向（id=3）
        assert_eq!(path, vec![1, 3], "平局必须走 node_id 较小的分支");
    }

    /// 机制 B 解析上界（ADR-NEW-20 三硬约束）：让步次数 ≤ ⌈log2(|V|/b)⌉+1 且必达终态。
    #[test]
    fn budget_yield_respects_analytic_bound() {
        let (net, coords) = grid(4, 4); // |V|=16
        let policy = BudgetPolicy { b: 1, g: 2 };
        let bound = policy.max_ticks_to_terminal(net.node_count());
        let mut pf = PathFinder::new(policy);
        let mut ticks = 0u64;
        loop {
            ticks += 1;
            match pf.request(&net, &coords, 0, 15) {
                FindOutcome::Complete { .. } => break,
                FindOutcome::Yielded { .. } => {
                    assert!(ticks <= bound, "超解析上界: ticks={ticks} bound={bound}");
                    continue;
                }
                FindOutcome::Unreachable => panic!("网格必连通"),
            }
        }
        assert!(ticks <= bound, "终态 tick 数必须 ≤ 解析上界");
        assert_eq!(pf.budget_yield_count, 0, "成功后让步计数清零");
        // B_k 递增性质
        assert_eq!(policy.budget_for_yield(0, 16), 1);
        assert_eq!(policy.budget_for_yield(1, 16), 2);
        assert_eq!(policy.budget_for_yield(4, 16), 16, "封顶 L=|V|");
    }

    /// 受阻终态：不可达必须返回 Unreachable（有限步终止，不得死循环）。
    #[test]
    fn unreachable_terminates_as_blocked() {
        let (mut net, coords) = grid(2, 2);
        // 删掉所有边 → 孤立
        for id in [1u64, 2, 3, 4] {
            net.publish(Tx { id: 100 + id, ops: vec![Op::RemoveEdge { id }] }).unwrap();
        }
        assert_eq!(find_once(&net, &coords, 0, 3, 100), FindOutcome::Unreachable);
    }

    /// 双计数器分离（ADR-NEW-20）：让步与过期重算互不影响。
    #[test]
    fn counters_are_separate() {
        let mut pf = PathFinder::new(BudgetPolicy { b: 1, g: 2 });
        pf.budget_yield_count = 5;
        pf.note_stale_recompute();
        pf.note_stale_recompute();
        assert_eq!(pf.budget_yield_count, 5, "过期重算不得动让步计数");
        assert_eq!(pf.stale_recompute_count, 2);
    }

    /// 平行同向边：路径边集必须是松弛实际使用的边（QWEN4-MAX 第二轮评审缺陷回归）。
    /// 回溯按 min edge id 会在平行边不同代价时返回与报告 cost 不符的边集。
    #[test]
    fn parallel_edge_reconstruction_is_exact() {
        let net = Network::from_edges(vec![
            Edge { id: 1, a: 0, b: 1, cost: 10, geom: vec![(0, 0), (0, 0)] }, // 平行边（较贵，id 小）
            Edge { id: 2, a: 0, b: 1, cost: 3, geom: vec![(0, 0), (0, 0)] },  // 平行边（较便宜）
            Edge { id: 3, a: 1, b: 2, cost: 5, geom: vec![(0, 0), (0, 0)] },
        ]);
        let coords = vec![(0i64, 0i64); 3];
        let FindOutcome::Complete { path, cost, .. } =
            find_once(&net, &coords, 0, 2, 100)
        else {
            panic!("must complete")
        };
        assert_eq!(cost, 8, "最优代价=3+5");
        assert_eq!(path, vec![2, 3], "必须选便宜的平行边 id=2，而非 min id=1");
        // 边集代价之和 == 报告 cost（stale 窄判依赖边集正确性）
        let sum: i64 = path
            .iter()
            .map(|&eid| net.edge(eid).expect("边存在").cost)
            .sum();
        assert_eq!(sum, cost, "返回边集的代价和必须等于报告 cost");
    }

    /// 溢出契约（ADR-NEW-21 决策 3）：饱和运算，禁止静默环绕；超大坐标/代价下确定性。
    #[test]
    fn overflow_saturates_deterministically() {
        let net = Network::from_edges(vec![Edge {
            id: 1,
            a: 0,
            b: 1,
            cost: i64::MAX,
            geom: vec![(0, 0), (0, 0)],
        }]);
        let coords = vec![(i64::MIN, i64::MIN), (i64::MAX, i64::MAX)];
        // h 不得环绕为负/恐慌：饱和到 i64::MAX
        assert_eq!(h_manhattan(&coords, 0, 1), i64::MAX);
        let a = find_once(&net, &coords, 0, 1, 100);
        let b = find_once(&net, &coords, 0, 1, 100);
        assert_eq!(a, b, "溢出路径下两次运行必须逐边一致");
        let FindOutcome::Complete { cost, path, .. } = a else { panic!("must complete") };
        assert!(cost >= 0, "饱和运算不得环绕为负");
        assert_eq!(path, vec![1]);
    }

    /// 债务 #29：全局配额截断——多请求共享 TickBudget，总展开 ≤ Q；耗尽后让步；reset 后恢复。
    #[test]
    fn global_budget_caps_total_expansions_per_tick() {
        let (net, coords) = grid(10, 10);
        let mut finder = PathFinder::new(BudgetPolicy::default());
        let mut budget = TickBudget::new(64);
        let mut total: u64 = 0;
        let mut completes = 0u32;
        let mut zero_yield = 0u32;
        for _ in 0..10 {
            let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
            match &out {
                FindOutcome::Complete { expanded, .. } => {
                    total += expanded;
                    completes += 1;
                }
                FindOutcome::Yielded { expanded } => {
                    total += expanded;
                    if *expanded == 0 {
                        zero_yield += 1;
                    }
                }
                FindOutcome::Unreachable => panic!("10x10 网格角到角必须可达"),
            }
        }
        assert!(total <= 64, "全局每 tick 总展开必须 ≤ Q，实测 {total}");
        assert!(completes >= 1, "配额内应有请求完成");
        assert!(zero_yield >= 1, "配额耗尽后请求必须以 expanded:0 让步");
        assert_eq!(budget.remaining, 0, "配额应恰好耗尽");
        // 下一 tick 重置后可以重新工作
        budget.reset();
        let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
        assert!(matches!(out, FindOutcome::Complete { .. }), "reset 后应完成");
    }

    /// 机制 B 推进：让步后下一 tick 以更大 B_k 重试，有限 tick 内完成（债务 #23/#29 交叉）。
    #[test]
    fn yield_then_fresh_budget_completes() {
        let (net, coords) = grid(10, 10);
        let mut finder = PathFinder::new(BudgetPolicy { b: 1, g: 2 });
        let mut budget = TickBudget::new(1024);
        let mut ticks = 0u64;
        loop {
            ticks += 1;
            budget.reset(); // 每 tick 预算独立
            let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
            match out {
                FindOutcome::Complete { .. } => break,
                FindOutcome::Yielded { .. } => assert!(ticks <= 16, "让步必须在有限 tick 内终止"),
                FindOutcome::Unreachable => panic!("必须可达"),
            }
        }
        assert_eq!(finder.budget_yield_count, 0, "完成后让步计数清零");
    }
}
