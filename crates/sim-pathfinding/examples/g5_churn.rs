//! g5_churn —— Stage 2A 退出判据「重算率 ≤ 5%」实测（churn 场景）。
//!
//! 场景：200×200 网格，200 个寻路请求（目标由 SplitMix64 确定性派生），
//! 每 20 个请求发布 1 笔拓扑事务（移除对角带 5 条边——覆盖主流路径的过期来源），
//! 每笔事务后对所有存活结果做 ADR-NEW-23 两阶段 stale_check：
//! Recompute → 记账一次重算并以机制 B 重找（新 base=当前 version）。
//! 输出：总检查数/重算数/重算率 + 阻塞率 + 假前向进展。
//!
//! 运行：cargo run --release --example g5_churn -p sim-pathfinding
//! 本文件为非权威测量工具（浮点比率按 clippy-layering-REPORT 豁免登记口径）。

#![allow(clippy::disallowed_types)]

use sim_network::{Edge, Network, Op, StaleOutcome, Tx};
use sim_pathfinding::{BudgetPolicy, FindOutcome, PathFinder};

const W: usize = 200;
const H: usize = 200;

fn grid_edges(w: usize, h: usize) -> Vec<Edge> {
    // 双向网格：每条链接两条有向边（a→b 与 b→a），保证任意两点互达（churn 场景需要）。
    let mut es = Vec::new();
    let mut id = 0u64;
    let push = |es: &mut Vec<Edge>, a: u64, b: u64, x1: i64, y1: i64, x2: i64, y2: i64, id: &mut u64| {
        es.push(Edge { id: *id, a, b, cost: 1000, geom: vec![(x1, y1), (x2, y2)] });
        *id += 1;
    };
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                let (a, b) = ((y * w + x) as u64, (y * w + x + 1) as u64);
                push(&mut es, a, b, (x * 1000) as i64, (y * 1000) as i64, ((x + 1) * 1000) as i64, (y * 1000) as i64, &mut id);
                push(&mut es, b, a, ((x + 1) * 1000) as i64, (y * 1000) as i64, (x * 1000) as i64, (y * 1000) as i64, &mut id);
            }
            if y + 1 < h {
                let (a, b) = ((y * w + x) as u64, ((y + 1) * w + x) as u64);
                push(&mut es, a, b, (x * 1000) as i64, (y * 1000) as i64, (x * 1000) as i64, ((y + 1) * 1000) as i64, &mut id);
                push(&mut es, b, a, (x * 1000) as i64, ((y + 1) * 1000) as i64, (x * 1000) as i64, (y * 1000) as i64, &mut id);
            }
        }
    }
    es
}

fn coords(w: usize, h: usize) -> Vec<(i64, i64)> {
    (0..w * h).map(|i| (((i % w) * 1000) as i64, ((i / w) * 1000) as i64)).collect()
}

fn splitmix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn main() {
    let n_nodes = W * H;
    let mut net = Network::from_edges(grid_edges(W, H));
    let coords = coords(W, H);
    let mut pf = PathFinder::new(BudgetPolicy::default());
    let mut tx_alloc = 0u64;

    let n_requests = 200u64;
    let tx_every = 20u64;

    let mut checks = 0u64;
    let mut recomputes = 0u64;
    let mut blocked = 0u64;
    let mut completed = 0u64;
    let mut total_ticks = 0u64;
    let mut worst_ticks = 0u64;

    // 存活结果：(request_id, path, base_version)
    let mut live: Vec<(u64, Vec<u64>, u64)> = Vec::new();

    for i in 0..n_requests {
        // 机制 B 求解（每次让步重做直到终态）
        let goal = splitmix64(i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7)) % (n_nodes as u64);
        let start = if i % 2 == 0 { 0u64 } else { (n_nodes - 1) as u64 };
        let mut ticks_used = 0u64;
        let path = loop {
            ticks_used += 1;
            match pf.request(&net, &coords, start, goal) {
                FindOutcome::Complete { path, .. } => break path,
                FindOutcome::Yielded { .. } => continue,
                FindOutcome::Unreachable { .. } => break Vec::new(),
            }
        };
        total_ticks += ticks_used;
        worst_ticks = worst_ticks.max(ticks_used);
        if path.is_empty() {
            blocked += 1;
        } else {
            completed += 1;
            live.push((i, path, net.version()));
        }

        // 每 tx_every 个请求发布一笔拓扑事务：移除对角带 5 条边
        if (i + 1) % tx_every == 0 {
            tx_alloc += 1;
            let n_edges = grid_edges(W, H).len() as u64; // 双向网格边总数（确定性）
            let band = i % (n_edges / 4);
            let ops: Vec<Op> = (0..5u64)
                .map(|k| Op::RemoveEdge { id: (band * 2 + k * 977) % n_edges })
                .collect();
            let _ = net.publish(Tx { id: tx_alloc, ops });

            // 两阶段 stale_check 所有存活结果
            let mut kept = Vec::new();
            for (rid, path, base) in live.drain(..) {
                checks += 1;
                match net.stale_check(&path, base) {
                    StaleOutcome::Accept => kept.push((rid, path, base)),
                    StaleOutcome::Recompute { .. } => {
                        recomputes += 1;
                        pf.note_stale_recompute();
                        // 机制 B 重找（新 base）
                        let goal = splitmix64(rid.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7)) % (n_nodes as u64);
                        let start = if rid % 2 == 0 { 0u64 } else { (n_nodes - 1) as u64 };
                        loop {
                            match pf.request(&net, &coords, start, goal) {
                                FindOutcome::Complete { path: np, .. } => {
                                    kept.push((rid, np, net.version()));
                                    break;
                                }
                                FindOutcome::Yielded { .. } => continue,
                                FindOutcome::Unreachable { .. } => {
                                    blocked += 1;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            live = kept;
        }
    }

    let recompute_rate = if checks == 0 { 0.0 } else { recomputes as f64 / checks as f64 };
    println!("G5 churn: grid {W}x{H}, requests={n_requests}, tx every {tx_every} (remove 5 diagonal edges each)");
    println!("requests terminal: {completed} | blocked: {blocked} | blocked_rate: {:.2}", blocked as f64 / n_requests as f64);
    println!("stale checks: {checks} | recomputes: {recomputes} | recompute_rate: {:.4}", recompute_rate);
    println!("pathing ticks total: {total_ticks} | worst per request: {worst_ticks} | budget_yield_count: {}", pf.budget_yield_count);
    println!("stale_recompute_count: {}", pf.stale_recompute_count);
    println!("exit criterion (recompute_rate <= 0.05): {}", if recompute_rate <= 0.05 { "PASS" } else { "FAIL" });
}
