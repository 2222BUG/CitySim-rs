//! B3b 并发队列压测（回归测试版，Stage 2B 硬门禁"并发队列压测"）。
//!
//! 场景：多请求并发寻路 + 事务交错（每 8 tick 删 4 边）+ 双计数器分离。
//! - 并行只读：每 tick 对全部存活路径结果并行执行 ADR-NEW-23 两阶段 stale_check
//!   （快照读取 + 稳定键排序 + 串行提交——2.9 条 6）；
//! - 串行权威：事务发布与请求处理（TickBudget 全局配额、请求 ID 稳定序）；
//! - 预算轴（yield_events）与路网轴（stale_recomputes）分别计数、互不混用；
//! - 断言：1/2/8 线程 + 混沌注入摘要一致；全部请求有限 tick 内终态（前向进展）。
//!
//! 小配额 QUOTA=200 是刻意的：强制触发预算让步路径（机制 B 的 B_k 递增 + 让步重试）。

use std::collections::BTreeMap;

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_network::{Edge, Network, Op, StaleOutcome, Tx};
use sim_parallel::{run_parallel, Task};
use sim_pathfinding::{BudgetPolicy, FindOutcome, PathFinder, TickBudget};
use sim_rng::event_u64;
use sim_test_support::chaos::chaos_spin;

const SEED: u64 = 0x00C0_FFEE_1234_5678;
const W: u64 = 40;
const H: u64 = 40;
const REQUESTS: u64 = 120;
const TICKS_MAX: u64 = 400;
const QUOTA: u64 = 200; // 刻意压小：强制预算让步
const TX_EVERY: u64 = 8;
const TX_REMOVE: u64 = 4;

fn coord(node: u64) -> (i64, i64) {
    (((node % W) * 1000) as i64, ((node / W) * 1000) as i64)
}

/// 双向网格（Edge=有向边，双向=两条；g5_churn 事故教训）。返回 (网络, 坐标, 边数)。
fn grid() -> (Network, Vec<(i64, i64)>, u64) {
    let mut edges = Vec::new();
    let mut coords = Vec::new();
    for n in 0..W * H {
        coords.push(coord(n));
    }
    let mut id = 0u64;
    for y in 0..H {
        for x in 0..W {
            let a = y * W + x;
            for b in [if x + 1 < W { Some(a + 1) } else { None }, if y + 1 < H { Some(a + W) } else { None }] {
                let Some(b) = b else { continue };
                for (p, q) in [(a, b), (b, a)] {
                    edges.push(Edge {
                        id,
                        a: p,
                        b: q,
                        cost: 1000,
                        geom: vec![coords[p as usize], coords[q as usize]],
                    });
                    id += 1;
                }
            }
        }
    }
    (Network::from_edges(edges), coords, id)
}

#[derive(Clone)]
struct Req {
    id: u64,
    start: u64,
    goal: u64,
}

#[derive(Clone)]
struct Live {
    start: u64,
    goal: u64,
    base: u64,
    edges: Vec<u64>,
}

/// stale 查询任务输入：(path_id, base_version, path_edges, chaos_seed)。
type StaleIn = (u64, u64, Vec<u64>, u64);

fn stale_task(net: &Network, inp: &StaleIn) -> StaleOutcome {
    let _ = chaos_spin(inp.0, inp.3);
    net.stale_check(&inp.2, inp.1)
}

#[derive(Default, Debug)]
struct Metrics {
    completed: u64,
    blocked_unreachable: u64,
    yield_events: u64,
    stale_recomputes: u64,
    tx_published: u64,
    tx_rejected: u64,
    ticks: u64,
    pending_left: u64,
}

fn run(threads: usize, chaos_seed: u64) -> (String, Metrics) {
    let (mut net, coords, n_edges) = grid();
    let mut pending: Vec<Req> = (0..REQUESTS)
        .map(|r| {
            let a = event_u64(SEED, "stress", r, 0) % (W * H);
            let mut b = event_u64(SEED, "stress", r, 1) % (W * H);
            if b == a {
                b = (b + 1) % (W * H);
            }
            Req {
                id: r + 1,
                start: a,
                goal: b,
            }
        })
        .collect();
    pending.sort_by_key(|r| r.id);
    let mut live: BTreeMap<u64, Live> = BTreeMap::new();
    let mut finder = PathFinder::new(BudgetPolicy::default());
    let mut budget = TickBudget::new(QUOTA);
    let mut m = Metrics::default();
    let mut gd = GlobalDigest::new();

    for tick in 1..=TICKS_MAX {
        m.ticks = tick;
        budget.reset();

        // (1) 并行只读：存活路径的过期查询（快照=网络 + 路径集，提交=串行）
        let tasks: Vec<Task<Network, StaleIn, StaleOutcome>> = live
            .iter()
            .map(|(pid, l)| Task {
                key: (0, *pid),
                input: (*pid, l.base, l.edges.clone(), chaos_seed),
                func: stale_task,
            })
            .collect();
        let mut requeue: Vec<u64> = Vec::new();
        run_parallel(&net, tasks, threads, |k, out| {
            if let StaleOutcome::Recompute { .. } = out {
                requeue.push(k.1);
            }
        });
        for pid in requeue {
            if let Some(l) = live.remove(&pid) {
                // 路网轴：过期重算（与预算让步分离；沿用原请求 ID 按全序重排——债务 #28）
                m.stale_recomputes += 1;
                finder.note_stale_recompute();
                pending.push(Req {
                    id: pid,
                    start: l.start,
                    goal: l.goal,
                });
            }
        }

        // (2) 事务交错：每 TX_EVERY tick 删 TX_REMOVE 条确定性选边（单向删，churn 口径）
        if tick % TX_EVERY == 0 {
            let mut ops = Vec::new();
            for k in 0..TX_REMOVE {
                let eid = (event_u64(SEED, "tx", tick, k) % n_edges) as u64;
                ops.push(Op::RemoveEdge { id: eid });
            }
            let tx = Tx { id: tick, ops };
            if net.publish(tx).is_ok() {
                m.tx_published += 1;
            } else {
                m.tx_rejected += 1;
            }
        }

        // (3) 串行权威：请求处理（请求 ID 稳定序 + TickBudget 全局配额）
        pending.sort_by_key(|r| r.id);
        let mut still = Vec::new();
        for r in pending.drain(..) {
            match finder.request_with_budget(&net, &coords, r.start, r.goal, &mut budget) {
                FindOutcome::Complete { path, .. } => {
                    m.completed += 1;
                    live.insert(
                        r.id,
                        Live {
                            start: r.start,
                            goal: r.goal,
                            base: net.version(),
                            edges: path,
                        },
                    );
                }
                FindOutcome::Yielded { .. } => {
                    m.yield_events += 1;
                    still.push(r);
                }
                FindOutcome::Unreachable { .. } => {
                    m.blocked_unreachable += 1;
                }
            }
        }
        pending = still;

        // (4) 逐 tick 折叠摘要（权威状态快照）
        let mut w = CanonicalWriter::new("sim-parallel", "QueueTick");
        w.u64(tick);
        w.u64(net.version());
        w.u64(pending.len() as u64);
        w.u64(live.len() as u64);
        w.u64(m.completed);
        w.u64(m.stale_recomputes);
        w.u64(m.yield_events);
        for (pid, l) in &live {
            w.u64(*pid);
            w.u64(l.base);
            w.seq_len(l.edges.len());
            for e in &l.edges {
                w.u64(*e);
            }
        }
        gd.record("sim-parallel", "QueueTick", w.as_bytes());

        if pending.is_empty() {
            break;
        }
    }
    m.pending_left = pending.len() as u64;
    (gd.finalize().hex(), m)
}

#[test]
fn concurrent_queue_thread_and_chaos_independence() {
    let (d1, m1) = run(1, 0);
    let (d2, _) = run(2, 0);
    let (d8, _) = run(8, 0);
    let (d42, _) = run(4, 42);
    assert_eq!(d1, d2, "1/2 线程摘要必须一致");
    assert_eq!(d1, d8, "1/8 线程摘要必须一致");
    assert_eq!(d1, d42, "混沌注入不得影响结果");
    // 前向进展：全部请求有限 tick 内终态（完成或明确受阻），无活锁
    assert_eq!(m1.pending_left, 0, "全部请求必须终态（防活锁）");
    // 会计恒等式：终态事件数 = 请求数 + 过期重算次数（每笔重算恰好再终态一次）
    assert_eq!(
        m1.completed + m1.blocked_unreachable,
        REQUESTS + m1.stale_recomputes,
        "终态事件数必须等于请求数加重算数"
    );
}

#[test]
fn budget_and_stale_counters_are_separate() {
    let (_, m) = run(2, 0);
    // 小配额刻意触发预算让步；事务交错触发过期重算——两轴独立计数
    assert!(m.yield_events > 0, "QUOTA=200 应触发预算让步路径");
    // 双计数器语义：让步与重算各自独立累计（0 也是合法终值，但不得互相替代）
    assert_eq!(m.pending_left, 0);
    assert_eq!(
        m.completed + m.blocked_unreachable,
        REQUESTS + m.stale_recomputes
    );
    assert!(m.ticks <= TICKS_MAX, "必须在上限 tick 内终态");
}
