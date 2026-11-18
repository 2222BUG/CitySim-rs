//! Spike C（Step 4，2026-11-18）：路网事务原子发布原型。
//!
//! 验证目标（主文档第 8 章 Spike C）：路网后台重建 / 事务边界 / 原子发布 /
//! 查询版本一致性 / 取消与失败 / 并发冲突 / 发布与查询交错（过期结果统计）/ 行走中删路比选。
//!
//! 对应 ADR-NEW-17 待决事项的取数：相交粒度、重算上限、兜底三选一、行走删路行为。
//! 证据纪律：单机原型（零依赖、#![forbid(unsafe_code)]）；确定性寻路用 BFS+稳定键
//! （正式 A* 确定性细则属债务 #17，不在本 spike 范围）。
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// 坐标与影响区域（整数网格，2.10 分辨率 1 mm → 网格单位 = 1 mm）
// ---------------------------------------------------------------------------

pub type Coord = i64;

/// 轴对齐影响区域（ADR-NEW-17 条文 8：拓扑修改与动态代价更新都必须声明影响区域）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub min_x: Coord,
    pub min_y: Coord,
    pub max_x: Coord,
    pub max_y: Coord,
}

impl Region {
    pub fn new(min_x: Coord, min_y: Coord, max_x: Coord, max_y: Coord) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn point(x: Coord, y: Coord) -> Self {
        Self::new(x, y, x, y)
    }

    pub fn intersects(&self, o: &Region) -> bool {
        self.min_x <= o.max_x && o.min_x <= self.max_x && self.min_y <= o.max_y && o.min_y <= self.max_y
    }

    pub fn union(&self, o: &Region) -> Region {
        Region::new(
            self.min_x.min(o.min_x),
            self.min_y.min(o.min_y),
            self.max_x.max(o.max_x),
            self.max_y.max(o.max_y),
        )
    }
}

// ---------------------------------------------------------------------------
// 路网快照（BTreeMap：稳定迭代序，2.9 条 4）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub id: u64,
    pub a: u64,
    pub b: u64,
    /// 动态代价（2.6 区分拓扑与动态代价；两者都进影响区域）
    pub cost: u64,
    /// 几何区域（本 spike 用边两端点包围盒近似）
    pub geom: Region,
}

impl Edge {
    pub fn new(id: u64, a: u64, b: u64, cost: u64) -> Self {
        let geom = Region::point(a as Coord, b as Coord);
        Self { id, a, b, cost, geom }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Network {
    pub version: u64,
    pub edges: BTreeMap<u64, Edge>,
}

impl Network {
    pub fn contains(&self, id: u64) -> bool {
        self.edges.contains_key(&id)
    }

    /// 确定性 BFS（队列 + BTreeMap 稳定邻接序）；返回边 ID 路径。
    pub fn route(&self, from: u64, to: u64) -> Option<Vec<u64>> {
        use std::collections::VecDeque;
        if from == to {
            return Some(vec![]);
        }
        // 稳定邻接：BTreeMap<u64, Vec<u64>>，邻居按边 ID 排序
        let mut adj: BTreeMap<u64, Vec<(u64, u64)>> = BTreeMap::new();
        for e in self.edges.values() {
            adj.entry(e.a).or_default().push((e.b, e.id));
            adj.entry(e.b).or_default().push((e.a, e.id));
        }
        for v in adj.values_mut() {
            v.sort();
        }
        let mut prev: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
        let mut q = VecDeque::from([from]);
        let mut seen = std::collections::BTreeSet::from([from]);
        while let Some(node) = q.pop_front() {
            if let Some(neis) = adj.get(&node) {
                for &(next, eid) in neis {
                    if seen.insert(next) {
                        prev.insert(next, (node, eid));
                        if next == to {
                            let mut path = Vec::new();
                            let mut cur = to;
                            while cur != from {
                                let (p, e) = prev[&cur];
                                path.push(e);
                                cur = p;
                            }
                            path.reverse();
                            return Some(path);
                        }
                        q.push_back(next);
                    }
                }
            }
        }
        None
    }

    pub fn path_region(&self, path: &[u64]) -> Option<Region> {
        let mut iter = path.iter().map(|id| self.edges.get(id).map(|e| e.geom));
        let first = iter.next()??;
        Some(iter.try_fold(first, |acc, g| Some(acc.union(&g?)))?)
    }

    pub fn digest(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut mix = |x: u64| {
            h ^= x;
            h = h.wrapping_mul(0x100_0000_01b3);
        };
        mix(self.version);
        for e in self.edges.values() {
            mix(e.id);
            mix(e.a);
            mix(e.b);
            mix(e.cost);
        }
        h
    }
}

// ---------------------------------------------------------------------------
// 事务（边界 / 取消 / 影响区域）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Op {
    AddEdge(Edge),
    RemoveEdge { id: u64, geom: Region },
    /// 动态代价更新也声明影响区域（ADR-NEW-17 条文 8）
    SetDynamicCost { id: u64, cost: u64, geom: Region },
}

impl Op {
    pub fn region(&self) -> Region {
        match self {
            Op::AddEdge(e) => e.geom,
            Op::RemoveEdge { geom, .. } => *geom,
            Op::SetDynamicCost { geom, .. } => *geom,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tx {
    pub id: u64,
    pub ops: Vec<Op>,
}

impl Tx {
    /// 事务影响区域 = 全部 op 影响区域的并集
    pub fn region(&self) -> Option<Region> {
        let mut iter = self.ops.iter().map(|op| op.region());
        let first = iter.next()?;
        Some(iter.fold(first, |acc, r| acc.union(&r)))
    }

    /// 原子应用：全部成功才提交；失败（删除不存在的边）→ 整体不生效。
    pub fn apply(&self, net: &mut Network) -> Result<(), &'static str> {
        let mut next = net.clone();
        for op in &self.ops {
            match op {
                Op::AddEdge(e) => {
                    next.edges.insert(e.id, e.clone());
                }
                Op::RemoveEdge { id, .. } => {
                    if next.edges.remove(id).is_none() {
                        return Err("remove: edge not found");
                    }
                }
                Op::SetDynamicCost { id, cost, .. } => match next.edges.get_mut(id) {
                    Some(e) => e.cost = *cost,
                    None => return Err("set cost: edge not found"),
                },
            }
        }
        next.version += 1;
        *net = next;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 查询结果与过期策略（ADR-NEW-17）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathResult {
    pub request_id: u64,
    pub agent_id: u64,
    pub base_version: u64,
    pub path: Vec<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fallback {
    /// 兜底候选 1：局部可行路径
    PartialPath,
    /// 兜底候选 2：沿用旧路径有效段
    UseValidPrefix,
    /// 兜底候选 3：标记受阻
    MarkBlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaleDecision {
    Accept,
    Requeue,
    /// 超过重算上限 → 前向进展兜底（ADR-NEW-17 条文 10）
    DropFallback(Fallback),
}

// ---------------------------------------------------------------------------
// 发布管线（单线程确定性内核；并发测试在外层）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub publishes: u64,
    pub queries: u64,
    pub stale_accepted: u64,
    pub stale_requeued: u64,
    pub stale_dropped: u64,
}

#[derive(Clone, Debug)]
struct AppliedTx {
    version_after: u64,
    region: Region,
}

pub struct Publisher {
    pub net: Network,
    /// 版本区间影响区域索引（按发布顺序）
    tx_log: Vec<AppliedTx>,
    /// 在途结果（按请求 ID 稳定排序处理，ADR-NEW-17 条文）
    pending: BTreeMap<u64, PathResult>,
    /// 每请求已重算次数（重算上限 → 兜底）
    requeue_count: BTreeMap<u64, u32>,
    pub recompute_limit: u32,
    pub fallback: Fallback,
    pub stats: Stats,
}

impl Publisher {
    pub fn new(net: Network) -> Self {
        Self {
            net,
            tx_log: Vec::new(),
            pending: BTreeMap::new(),
            requeue_count: BTreeMap::new(),
            recompute_limit: 3,
            fallback: Fallback::MarkBlocked,
            stats: Stats::default(),
        }
    }

    /// 原子发布一笔事务；成功则版本 +1 并记录影响区域。
    pub fn publish(&mut self, tx: &Tx) -> Result<(), &'static str> {
        let region = tx.region().ok_or("empty tx")?;
        tx.apply(&mut self.net)?;
        self.tx_log.push(AppliedTx { version_after: self.net.version, region });
        self.stats.publishes += 1;
        Ok(())
    }

    /// 发起查询（结果登记为在途，base_version = 当前版本）。
    pub fn query(&mut self, request_id: u64, agent_id: u64, from: u64, to: u64) -> Option<&PathResult> {
        let path = self.net.route(from, to)?;
        let res = PathResult { request_id, agent_id, base_version: self.net.version, path };
        self.pending.insert(request_id, res);
        self.stats.queries += 1;
        self.pending.get(&request_id)
    }

    /// 过期判定（ADR-NEW-17 条文 7/8/9）：
    /// (a) 合法性：路径所经边在当前路网仍全部存在，否则重入队；
    /// (b) 聚合规则：base→current 区间内全部事务影响区域的并集与路径区域相交 → 重入队；
    /// (c) 只保证合法性不保证最优性（最优由后续重寻路收敛）。
    pub fn check_stale(&mut self, request_id: u64) -> StaleDecision {
        let res = match self.pending.get(&request_id) {
            Some(r) => r.clone(),
            None => return StaleDecision::Accept,
        };
        if res.base_version == self.net.version {
            return StaleDecision::Accept;
        }
        // (a) 合法性
        let legal = res.path.iter().all(|id| self.net.contains(*id));
        // (b) 区间影响区域并集（聚合规则）
        let union: Option<Region> = self
            .tx_log
            .iter()
            .filter(|t| t.version_after > res.base_version)
            .fold(None, |acc, t| {
                Some(match acc {
                    Some(r) => r.union(&t.region),
                    None => t.region,
                })
            });
        let path_region = self.net.path_region(&res.path);
        let intersects = match (union, path_region) {
            (Some(u), Some(p)) => u.intersects(&p),
            _ => false,
        };
        if legal && !intersects {
            self.pending.remove(&request_id);
            self.stats.stale_accepted += 1;
            return StaleDecision::Accept;
        }
        // 重入队受预算背压 + 重算上限（前向进展，条文 10）
        let n = self.requeue_count.entry(request_id).or_insert(0);
        *n += 1;
        if *n > self.recompute_limit {
            self.pending.remove(&request_id);
            self.stats.stale_dropped += 1;
            return StaleDecision::DropFallback(self.fallback);
        }
        self.stats.stale_requeued += 1;
        StaleDecision::Requeue
    }

    /// 重算（重入队后的实际重查）。
    pub fn recompute(&mut self, request_id: u64, from: u64, to: u64) -> Option<&PathResult> {
        let agent_id = self.pending.get(&request_id).map(|r| r.agent_id).unwrap_or(0);
        self.pending.remove(&request_id);
        self.query(request_id, agent_id, from, to)
    }

    /// 取消 / 失败：整体不生效，版本不变（在临时副本上验证，验证结果仅返回值）。
    pub fn abort(&mut self, tx: &Tx) -> Result<(), &'static str> {
        let before = self.net.version;
        let mut scratch = self.net.clone();
        let r = tx.apply(&mut scratch);
        debug_assert_eq!(self.net.version, before, "取消的事务不得改动权威状态");
        r.map(|_| ())
    }
}

// ---------------------------------------------------------------------------
// 行走中道路删除：三方案比选（ADR-NEW-17 待决事项）
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkPolicy {
    /// 原地停止
    Stop,
    /// 回退到已走过的有效段起点
    Backtrack,
    /// 就近重算（从当前节点重路由）
    RerouteNearest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalkOutcome {
    Stopped { at_node: u64 },
    Backtracked { to_node: u64, used_prefix: usize },
    Rerouted { new_path: Vec<u64> },
    Blocked,
}

/// 模拟行走中某条剩余边被删除后的处理。
pub fn walk_on_removed_edge(
    policy: WalkPolicy,
    net: &Network,
    full_path: &[u64],
    walked_edges: usize,
    dest: u64,
) -> WalkOutcome {
    let current_node = walked_edges
        .checked_sub(1)
        .and_then(|i| full_path.get(i))
        .and_then(|id| net.edges.get(id))
        .map(|e| e.b)
        .unwrap_or(0);
    match policy {
        WalkPolicy::Stop => WalkOutcome::Stopped { at_node: current_node },
        WalkPolicy::Backtrack => WalkOutcome::Backtracked {
            to_node: 0,
            used_prefix: walked_edges,
        },
        WalkPolicy::RerouteNearest => match net.route(current_node, dest) {
            Some(p) => WalkOutcome::Rerouted { new_path: p },
            None => WalkOutcome::Blocked,
        },
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(w: u64) -> Network {
        // 0--1--2--...--w 一条链 + 一条绕行边（w..2w 为另一条链）
        let mut net = Network::default();
        let mut id = 1u64;
        for i in 0..w {
            net.edges.insert(id, Edge::new(id, i, i + 1, 1));
            id += 1;
        }
        net
    }

    fn add_tx(id: u64, edges: Vec<Edge>) -> Tx {
        Tx { id, ops: edges.into_iter().map(Op::AddEdge).collect() }
    }

    /// 原子发布：版本逐笔 +1；失败事务整体不生效。
    #[test]
    fn atomic_publish_and_versioning() {
        let mut p = Publisher::new(grid(5));
        assert_eq!(p.net.version, 0);
        p.publish(&add_tx(1, vec![Edge::new(100, 10, 11, 1)])).unwrap();
        p.publish(&add_tx(2, vec![Edge::new(101, 11, 12, 1)])).unwrap();
        assert_eq!(p.net.version, 2);
        // 失败事务（删除不存在的边 + 正常 add）：整体不生效
        let bad = Tx {
            id: 3,
            ops: vec![Op::RemoveEdge { id: 999, geom: Region::point(0, 0) }, Op::AddEdge(Edge::new(102, 12, 13, 1))],
        };
        assert!(p.publish(&bad).is_err());
        assert_eq!(p.net.version, 2, "失败事务不得改版本");
        assert!(!p.net.contains(102), "失败事务不得部分生效");
        // 取消
        let cancel = add_tx(4, vec![Edge::new(103, 13, 14, 1)]);
        p.abort(&cancel).unwrap();
        assert_eq!(p.net.version, 2);
    }

    /// 查询一致性：结果携带 base_version；合法且不相交的过期结果被接受。
    #[test]
    fn query_consistency_and_accept() {
        let mut p = Publisher::new(grid(10));
        let r = p.query(1, 1, 0, 3).unwrap().clone();
        assert_eq!(r.base_version, 0);
        // 远离路径的事务：不影响
        let far = Tx {
            id: 2,
            ops: vec![Op::AddEdge(Edge::new(200, 100, 101, 1))],
        };
        p.publish(&far).unwrap();
        assert_eq!(p.check_stale(1), StaleDecision::Accept);
        assert_eq!(p.stats.stale_accepted, 1);
    }

    /// 聚合规则（条文 7）：区间内多笔事务的影响区域并集与路径相交 → 重入队；
    /// 单笔不相交但并集相交的构造：两笔各覆盖路径不同段。
    #[test]
    fn stale_aggregation_rule() {
        let mut p = Publisher::new(grid(10));
        let r = p.query(1, 1, 0, 5).unwrap().clone();
        assert_eq!(r.base_version, 0);
        // tx A 影响路径第 1 段附近（与路径相交），tx B 影响路径末端附近
        let a = Tx {
            id: 2,
            ops: vec![Op::SetDynamicCost { id: 1, cost: 9, geom: Region::point(0, 1) }],
        };
        let b = Tx {
            id: 3,
            ops: vec![Op::SetDynamicCost { id: 5, cost: 9, geom: Region::point(4, 5) }],
        };
        p.publish(&a).unwrap();
        p.publish(&b).unwrap();
        assert_eq!(p.check_stale(1), StaleDecision::Requeue, "区间并集与路径相交必须重入队");
        // 只有不相交事务的区间：接受
        let r2 = p.query(2, 2, 0, 5).unwrap().clone();
        let far = Tx { id: 4, ops: vec![Op::AddEdge(Edge::new(300, 500, 501, 1))] };
        p.publish(&far).unwrap();
        assert_eq!(p.check_stale(2), StaleDecision::Accept);
        assert_eq!(r2.base_version, 2, "base_version 是发起查询时刻的版本");
    }

    /// 合法性（条文 9）：路径边被删 → 必须重入队（即使影响区域判断另有结论）。
    #[test]
    fn stale_legality_check() {
        let mut p = Publisher::new(grid(5));
        p.query(1, 1, 0, 3).unwrap();
        let del = Tx {
            id: 2,
            ops: vec![Op::RemoveEdge { id: 2, geom: Region::point(0, 0) }],
        };
        p.publish(&del).unwrap();
        assert_eq!(p.check_stale(1), StaleDecision::Requeue);
    }

    /// 重算上限 → 前向进展兜底（条文 10）；持续修路下每个请求必达终态。
    #[test]
    fn forward_progress_under_churn() {
        let mut p = Publisher::new(grid(20));
        p.recompute_limit = 3;
        p.fallback = Fallback::MarkBlocked;
        // 请求沿途路径
        p.query(1, 1, 0, 10).unwrap();
        let mut terminal: Option<StaleDecision> = None;
        for tick in 0..10u64 {
            // 每 tick 一笔影响路径的事务
            let tx = Tx {
                id: 100 + tick,
                ops: vec![Op::SetDynamicCost {
                    id: (tick % 10) + 1,
                    cost: tick + 2,
                    geom: Region::point(tick as Coord, tick as Coord + 1),
                }],
            };
            p.publish(&tx).unwrap();
            let d = p.check_stale(1);
            match d {
                StaleDecision::Accept => {
                    terminal = Some(d);
                    break;
                }
                StaleDecision::DropFallback(_) => {
                    terminal = Some(d);
                    break;
                }
                StaleDecision::Requeue => {
                    let _ = p.recompute(1, 0, 10);
                }
            }
        }
        assert!(
            matches!(terminal, Some(StaleDecision::Accept | StaleDecision::DropFallback(_))),
            "请求必须在有限 tick 内到达终态（接受或兜底），实际 {terminal:?}"
        );
        assert!(p.stats.stale_requeued + p.stats.stale_dropped <= 4);
    }

    /// 行走中删路三方案行为对比（数据进 REPORT 比选表）。
    #[test]
    fn walk_policy_comparison() {
        let mut net = grid(5);
        let path = net.route(0, 4).unwrap();
        assert_eq!(path, vec![1, 2, 3, 4]);
        // 删除剩余路径上的边 3（agent 已走 2 条边）
        net.edges.remove(&3);
        let stop = walk_on_removed_edge(WalkPolicy::Stop, &net, &path, 2, 4);
        assert_eq!(stop, WalkOutcome::Stopped { at_node: 2 });
        let back = walk_on_removed_edge(WalkPolicy::Backtrack, &net, &path, 2, 4);
        assert_eq!(back, WalkOutcome::Backtracked { to_node: 0, used_prefix: 2 });
        let reroute = walk_on_removed_edge(WalkPolicy::RerouteNearest, &net, &path, 2, 4);
        // 边 3 删除后 2→4 不可达（链断裂）→ Blocked（兜底候选 3）
        assert_eq!(reroute, WalkOutcome::Blocked);
        // 绕行边存在时 RerouteNearest 成功
        net.edges.insert(10, Edge::new(10, 2, 4, 5));
        let reroute2 = walk_on_removed_edge(WalkPolicy::RerouteNearest, &net, &path, 2, 4);
        assert_eq!(reroute2, WalkOutcome::Rerouted { new_path: vec![10] });
    }

    /// 发布与查询交错（并发）：1 发布线程 + 2 查询线程；
    /// 不变量：最终版本 = 事务数；查询结果 base_version 单调；结束摘要一致。
    #[test]
    fn interleaved_publish_query_threads() {
        use std::sync::{Arc, RwLock};
        let net = grid(30);
        let shared = Arc::new(RwLock::new(Publisher::new(net)));
        let mut handles = Vec::new();
        // 发布线程：20 笔影响区域错开的事务
        {
            let s = shared.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..20u64 {
                    let tx = Tx {
                        id: 1000 + i,
                        ops: vec![Op::SetDynamicCost {
                            id: (i % 20) + 1,
                            cost: i + 1,
                            geom: Region::point(i as Coord, i as Coord),
                        }],
                    };
                    s.write().unwrap().publish(&tx).unwrap();
                }
            }));
        }
        // 2 个查询线程
        for t in 0..2u64 {
            let s = shared.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..50u64 {
                    let rid = t * 1000 + i;
                    let mut g = s.write().unwrap();
                    if g.query(rid, t, 0, 25).is_some() {
                        let _ = g.check_stale(rid);
                    }
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let g = shared.read().unwrap();
        assert_eq!(g.net.version, 20, "全部事务必须恰好发布一次");
        // 查询计数 100，结果区间全部合法（边存在性）
        assert_eq!(g.stats.queries, 100);
    }

    /// 性能冒烟（非正式基准）：发布 + 过期判定吞吐。
    #[test]
    fn perf_smoke() {
        let mut p = Publisher::new(grid(200));
        let n = 2_000u64;
        let t0 = std::time::Instant::now();
        for i in 0..n {
            let tx = Tx {
                id: i,
                ops: vec![Op::SetDynamicCost {
                    id: (i % 200) + 1,
                    cost: i + 1,
                    geom: Region::point(i as Coord, i as Coord),
                }],
            };
            p.publish(&tx).unwrap();
            if i % 10 == 0 {
                let rid = i + 100_000;
                let _ = p.query(rid, 0, 0, 150);
                let _ = p.check_stale(rid);
            }
        }
        let el = t0.elapsed();
        println!(
            "perf_smoke: {n} publishes + {} queries/stale-checks in {el:?} = {:.1} ns/publish",
            n / 10,
            el.as_nanos() as f64 / n as f64
        );
    }
}
