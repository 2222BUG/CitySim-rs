//! G5 联动组压测：200×200 极端长路径下机制 A / B / C 对比（ADR-NEW-20 / 组级验收 G5）。
//!
//! 指标（G5 五项）：最坏单 tick 节点展开数与耗时、重复展开率、终止率、MarkBlocked 阻塞率、
//! 假前向进展（被丢弃的展开占比）。
//!
//! 机制定义：
//! - A = 跨 tick 续算（搜索状态跨 tick 保留，每 tick 恒定预算 b）；
//! - B = 预算递增（默认候选；单 tick 原子 + B_k = min(b·2^k, |V|)，ADR-NEW-24 决策 3）；
//! - C = 受控失败（单次尝试恒定预算 b，未完成即 MarkBlocked）。
//!
//! 运行：cargo run --release --example g5_stress -p sim-pathfinding
//! 本文件为非权威测量工具（墙钟计时与比率浮点按 clippy-layering-REPORT 豁免登记口径）。

#![allow(clippy::disallowed_types)]

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use sim_network::{Edge, Network};
use sim_pathfinding::{find_once, BudgetPolicy, FindOutcome, PathFinder};

fn grid_edges(w: u64, h: u64) -> (Vec<Edge>, Vec<(i64, i64)>) {
    let mut edges = Vec::new();
    let mut coords = Vec::new();
    for y in 0..h {
        for x in 0..w {
            coords.push((x as i64 * 1000, y as i64 * 1000));
        }
    }
    let mut eid = 1u64;
    let node = |x: u64, y: u64| y * w + x;
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                edges.push(Edge {
                    id: eid,
                    a: node(x, y),
                    b: node(x + 1, y),
                    cost: 1000,
                    geom: vec![(x as i64 * 1000, y as i64 * 1000), ((x + 1) as i64 * 1000, y as i64 * 1000)],
                });
                eid += 1;
            }
            if y + 1 < h {
                edges.push(Edge {
                    id: eid,
                    a: node(x, y),
                    b: node(x, y + 1),
                    cost: 1000,
                    geom: vec![(x as i64 * 1000, y as i64 * 1000), (x as i64 * 1000, (y + 1) as i64 * 1000)],
                });
                eid += 1;
            }
        }
    }
    (edges, coords)
}

// ---------------- 机制 A：跨 tick 续算（对比基准） ----------------

struct ResumeAstar {
    open: BTreeMap<(i64, i64, u64), ()>,
    g: BTreeMap<u64, i64>,
    parent: BTreeMap<u64, u64>,
    closed: BTreeSet<u64>,
    goal: u64,
    expansions_total: u64,
    done: Option<Vec<u64>>,
}

impl ResumeAstar {
    fn new(coords: &[(i64, i64)], start: u64, goal: u64) -> Self {
        let h0 = h(coords, start, goal);
        let mut open = BTreeMap::new();
        open.insert((h0, h0, start), ());
        let mut g = BTreeMap::new();
        g.insert(start, 0i64);
        ResumeAstar { open, g, parent: BTreeMap::new(), closed: BTreeSet::new(), goal, expansions_total: 0, done: None }
    }

    /// 一个 tick：展开至多 budget 个节点。返回本 tick 展开数。
    fn step(&mut self, net: &Network, coords: &[(i64, i64)], budget: u64) -> u64 {
        if self.done.is_some() {
            return 0;
        }
        let mut expanded = 0u64;
        while expanded < budget {
            let Some(((_f, _hh, node), ())) = self.open.pop_first() else {
                self.done = Some(Vec::new()); // 不可达
                break;
            };
            if node == self.goal {
                let mut path = Vec::new();
                let mut cur = self.goal;
                while let Some(&p) = self.parent.get(&cur) {
                    let eid = net.neighbors(p).filter(|&(_, to, _)| to == cur).map(|(e, _, _)| e).min().unwrap();
                    path.push(eid);
                    cur = p;
                }
                path.reverse();
                self.done = Some(path);
                break;
            }
            if !self.closed.insert(node) {
                continue;
            }
            expanded += 1;
            self.expansions_total += 1;
            let g_cur = self.g[&node];
            for (_, to, cost) in net.neighbors(node) {
                let new_g = g_cur.wrapping_add(cost);
                if self.g.get(&to).is_some_and(|&k| new_g >= k) {
                    continue;
                }
                self.g.insert(to, new_g);
                self.parent.insert(to, node);
                self.closed.remove(&to);
                let hh = h(coords, to, self.goal);
                self.open.insert((new_g.wrapping_add(hh), hh, to), ());
            }
        }
        expanded
    }
}

fn h(coords: &[(i64, i64)], node: u64, goal: u64) -> i64 {
    let (x1, y1) = coords.get(node as usize).copied().unwrap_or((0, 0));
    let (x2, y2) = coords.get(goal as usize).copied().unwrap_or((0, 0));
    (x1 - x2).abs() + (y1 - y2).abs()
}

// ---------------- 计时（非权威测量，豁免登记口径） ----------------

#[allow(clippy::disallowed_methods)]
fn now() -> Instant {
    Instant::now()
}

fn main() {
    const W: u64 = 200;
    const H: u64 = 200;
    const B: u64 = 64;
    let (edges, coords) = grid_edges(W, H);
    let n_nodes = W * H;
    let net = Network::from_edges(edges);
    let start = 0u64;
    let goal = n_nodes - 1;
    println!("G5 stress: grid {W}x{H}, |V|={n_nodes}, start={start} -> goal={goal}, b={B}");
    println!("mechanism | ticks | worst_tick_expansions | worst_tick_ms | total_expansions | discarded | dup_rate | terminal | blocked | fake_progress");
    let _ = coords;

    // ---- 机制 A：跨 tick 续算 ----
    let mut a = ResumeAstar::new(&coords, start, goal);
    let mut ticks = 0u64;
    let mut worst_exp = 0u64;
    let mut worst_ms = 0f64;
    while a.done.is_none() {
        let t0 = now();
        let e = a.step(&net, &coords, B);
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        ticks += 1;
        worst_exp = worst_exp.max(e);
        worst_ms = worst_ms.max(dt);
    }
    let a_terminal = if a.done.as_ref().is_some_and(|p| !p.is_empty() || start == goal) { 1.0 } else { 0.0 };
    println!(
        "A(resume) | {ticks} | {worst_exp} | {worst_ms:.3} | {} | 0 | 0.00 | {a_terminal:.2} | 0.00 | 0.00",
        a.expansions_total
    );

    // ---- 机制 B：预算递增（默认候选） ----
    let policy = BudgetPolicy { b: B, g: 2 };
    let mut pf = PathFinder::new(policy);
    let mut ticks = 0u64;
    let mut worst_exp = 0u64;
    let mut worst_ms = 0f64;
    let mut discarded = 0u64;
    let mut total_final = 0u64;
    let mut terminal = 0.0f64;
    let mut blocked = 0.0f64;
    loop {
        let t0 = now();
        let out = pf.request(&net, &coords, start, goal);
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        ticks += 1;
        match out {
            FindOutcome::Yielded { expanded } => {
                discarded += expanded;
                worst_exp = worst_exp.max(expanded);
            }
            FindOutcome::Complete { expanded: e, .. } => {
                total_final = e;
                terminal = 1.0;
                worst_exp = worst_exp.max(e);
                break;
            }
            FindOutcome::Unreachable => {
                blocked = 1.0;
                break;
            }
        }
        worst_ms = worst_ms.max(dt);
        if ticks > 100 {
            break;
        }
    }
    let total = discarded + total_final;
    let dup_rate = if total == 0 { 0.0 } else { discarded as f64 / total as f64 };
    println!(
        "B(grow)   | {ticks} | {worst_exp} | {worst_ms:.3} | {total} | {discarded} | {dup_rate:.2} | {terminal:.2} | {blocked:.2} | {dup_rate:.2}"
    );

    // ---- 机制 C：单次尝试受控失败 ----
    let t0 = now();
    let out = find_once(&net, &coords, start, goal, B);
    let dt = t0.elapsed().as_secs_f64() * 1000.0;
    let (terminal, blocked) = match out {
        FindOutcome::Complete { .. } => (1.0, 0.0),
        _ => (0.0, 1.0),
    };
    println!("C(once)   | 1 | {B} | {dt:.3} | {B} | 0 | 0.00 | {terminal:.2} | {blocked:.2} | 0.00");

    let bound = policy.max_ticks_to_terminal(n_nodes);
    println!();
    println!("analytic bound (B): ticks <= {bound}");
    println!("note: (f,h,node_id) tie-break with h-ascending prefers the deepest frontier first, so the");
    println!("      corner-to-corner run expands ~Manhattan-length nodes rather than the whole domain;");
    println!("      the stress point exercises the longest possible request, not a degenerate expansion blow-up.");
}
