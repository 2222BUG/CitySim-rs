//! 债务 #27（ADR-NEW-24 切换阈值过渡口径）：机制 B→A 切换阈值多场景基线。
//!
//! 场景：open 200×200（复现 G5-stress 基线）/ 障碍 20%（确定性堵块）/ 迷宫（走廊）。
//! 每请求手写机制 B 循环（B_k=min(b·2^k, |V|)），逐请求记录
//!   dup_rate = discarded / (discarded + final_expansions)
//! 输出分布（mean/p95/max）、最坏单 tick 展开与耗时、完成/阻塞率。
//! 非权威测量工具（墙钟+比率浮点按 clippy-layering-REPORT 豁免登记口径）。

#![allow(clippy::disallowed_types)]
#![allow(clippy::disallowed_methods)]

use sim_network::{Edge, Network};
use sim_pathfinding::{find_once, FindOutcome};
use sim_rng::event_u64;
use std::time::Instant;

const W: u64 = 200;
const H: u64 = 200;
const SEED: u64 = 0x00C0_FFEE_1234_5678;
const REQUESTS: u64 = 200;
const B0: u64 = 64;
const G: u64 = 2;

fn coords_grid(w: u64, h: u64) -> Vec<(i64, i64)> {
    (0..w * h).map(|n| ((n % w) as i64 * 1000, (n / w) as i64 * 1000)).collect()
}

/// 双向网格；kept(edge_id)->false 剔除（确定性障碍；id 按剔除前位置递增，口径稳定）。
fn grid_edges(w: u64, h: u64, kept: &dyn Fn(u64) -> bool) -> Vec<Edge> {
    let mut edges = Vec::new();
    let mut id = 0u64;
    for i in 0..h {
        for j in 0..w {
            let n = i * w + j;
            let pairs: &[(u64, u64)] = if j + 1 < w && i + 1 < h {
                &[(n, n + 1), (n + 1, n), (n, n + w), (n + w, n)]
            } else if j + 1 < w {
                &[(n, n + 1), (n + 1, n)]
            } else if i + 1 < h {
                &[(n, n + w), (n + w, n)]
            } else {
                &[]
            };
            for &(a, b) in pairs {
                if kept(id) {
                    edges.push(Edge { id, a, b, cost: 1000, geom: vec![(0, 0), (0, 0)] });
                }
                id += 1;
            }
        }
    }
    edges
}

struct ReqStat {
    dup_rate: f64,
    total_expansions: u64,
    final_expansions: u64,
    yields: u64,
    completed: bool,
}

/// 机制 B：B_k=min(b·2^k,|V|) 跨 tick 让步续跑；丢弃展开=让步尝试的展开数。
fn run_request(net: &Network, coords: &[(i64, i64)], start: u64, goal: u64, n_nodes: u64) -> ReqStat {
    let mut discarded = 0u64;
    let mut k = 0u64;
    loop {
        let bk = (B0.saturating_mul(1u64 << k.min(20))).min(n_nodes);
        match find_once(net, coords, start, goal, bk) {
            FindOutcome::Complete { expanded, .. } => {
                let total = discarded + expanded;
                return ReqStat {
                    dup_rate: if total == 0 { 0.0 } else { discarded as f64 / total as f64 },
                    total_expansions: total,
                    final_expansions: expanded,
                    yields: k,
                    completed: true,
                };
            }
            FindOutcome::Yielded { expanded } => {
                discarded += expanded;
                k += 1;
            }
            FindOutcome::Unreachable => {
                return ReqStat {
                    dup_rate: if discarded == 0 { 0.0 } else { 1.0 },
                    total_expansions: discarded,
                    final_expansions: 0,
                    yields: k,
                    completed: false,
                };
            }
        }
    }
}

fn run_scenario(name: &str, net: &Network, coords: &[(i64, i64)], n_nodes: u64, tag: &str) {
    let mut dups = Vec::new();
    let mut worst_tick_ms = 0f64;
    let mut worst_expansions = 0u64;
    let mut worst_single_expansions = 0u64; // 单次 find_once 上界观测
    let mut worst_yields = 0u64;
    let mut completed = 0u64;
    let mut blocked = 0u64;
    for r in 0..REQUESTS {
        let s = event_u64(SEED, tag, r, 0) % n_nodes;
        let g = event_u64(SEED, tag, r, 1) % n_nodes;
        if s == g {
            continue;
        }
        let t0 = Instant::now();
        let st = run_request(net, coords, s, g, n_nodes);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        if ms > worst_tick_ms {
            worst_tick_ms = ms;
        }
        if st.total_expansions > worst_expansions {
            worst_expansions = st.total_expansions;
        }
        if st.final_expansions > worst_single_expansions {
            worst_single_expansions = st.final_expansions;
        }
        if st.yields > worst_yields {
            worst_yields = st.yields;
        }
        if st.completed {
            completed += 1;
            dups.push(st.dup_rate);
        } else {
            blocked += 1;
        }
    }
    dups.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = if dups.is_empty() { 0.0 } else { dups.iter().sum::<f64>() / dups.len() as f64 };
    let p95 = if dups.is_empty() { 0.0 } else { dups[(dups.len() * 95 / 100).min(dups.len() - 1)] };
    let max = dups.last().copied().unwrap_or(0.0);
    println!("[{name}] completed={completed} blocked={blocked}");
    println!("  dup_rate mean={mean:.3} p95={p95:.3} max={max:.3}");
    println!("  worst_request_expansions={worst_expansions} worst_single_attempt={worst_single_expansions} worst_yields={worst_yields} worst_wall_ms={worst_tick_ms:.3}");
}

fn main() {
    let n_nodes = W * H;
    let coords = coords_grid(W, H);
    println!("=== 债务 #27 阈值基线（机制 B: b={B0} g={G} L={n_nodes}）===");
    println!("requests per scenario={REQUESTS} seed=0x{SEED:016X}");

    // 场景 1：open（对照 G5-stress 基线 0.53）
    let open = Network::from_edges(grid_edges(W, H, &|_| true));
    run_scenario("open", &open, &coords, n_nodes, "thr.open");

    // 场景 2：障碍 20%（确定性：event_u64 % 5 == 0 剔除）
    let obs = Network::from_edges(grid_edges(W, H, &|eid| !event_u64(SEED, "thr.obs", eid, 0).is_multiple_of(5)));
    run_scenario("obstacle-20pct", &obs, &coords, n_nodes, "thr.obs");

    // 场景 3：迷宫（奇数行全墙，每 10 列留口——走廊迫使长绕行）
    let maze = Network::from_edges(grid_edges(W, H, &|eid| {
        // eid 顺序：行内每格先 (n→n+1, n+1→n) 再 (n→n+w, n+w→n)；
        // 近似判据：按 eid%4 与行号奇偶过滤竖边（简化为确定性 3/4 保留奇数行竖边删除）
        let row = eid / (W * 2);
        if row % 2 == 1 {
            // 奇数行：竖边全删（留口由横向间隙形成）
            eid % 4 >= 2
        } else {
            true
        }
    }));
    run_scenario("maze-corridor", &maze, &coords, n_nodes, "thr.maze");

    println!("=== 判定口径参考：MiMo 候选=worst_tick>1ms 或 dup>0.7；Grok=dup≥1.0；Gemini=P95>0.65 ===");
}
