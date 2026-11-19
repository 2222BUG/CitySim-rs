//! B3b 并发队列压测（Stage 2B 硬门禁"并发队列压测"实测数据采集）。
//!
//! 大规模场景：200×200 双向网格（40k 节点）、500 请求、事务交错 churn、
//! TickBudget 全局配额 65536/ tick。扫 1/2/8 线程 + 混沌注入，
//! 验证摘要一致并记录墙钟与双计数器指标 → docs/benchmarks/Stage2B-concurrent-stress.txt。
//!
//! 运行：cargo run --release --example concurrent_stress -p sim-pathfinding
//! 本文件为非权威测量工具（墙钟计时与比率浮点按 clippy-layering-REPORT 豁免登记口径）。

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::time::Instant;

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_network::{Edge, Network, Op, StaleOutcome, Tx};
use sim_parallel::{run_parallel, Task};
use sim_pathfinding::{BudgetPolicy, FindOutcome, PathFinder, TickBudget};
use sim_rng::event_u64;
use sim_test_support::chaos::chaos_spin;

const SEED: u64 = 0x00C0_FFEE_1234_5678;
const W: u64 = 200;
const H: u64 = 200;
const REQUESTS: u64 = 500;
const TICKS_MAX: u64 = 300;
const QUOTA: u64 = 65536;
const TX_EVERY: u64 = 8;
const TX_REMOVE: u64 = 4;

fn coord(node: u64) -> (i64, i64) {
    (((node % W) * 1000) as i64, ((node / W) * 1000) as i64)
}

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
            for b in [
                if x + 1 < W { Some(a + 1) } else { None },
                if y + 1 < H { Some(a + W) } else { None },
            ] {
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
    stale_sweeps: u64,
    wall_ms: f64,
}

fn run(threads: usize, chaos_seed: u64) -> (String, Metrics) {
    let t0 = Instant::now();
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

        // (1) 并行只读过期查询（快照+稳定键+串行提交）
        let tasks: Vec<Task<Network, StaleIn, StaleOutcome>> = live
            .iter()
            .map(|(pid, l)| Task {
                key: (0, *pid),
                input: (*pid, l.base, l.edges.clone(), chaos_seed),
                func: stale_task,
            })
            .collect();
        m.stale_sweeps += tasks.len() as u64;
        let mut requeue: Vec<u64> = Vec::new();
        run_parallel(&net, tasks, threads, |k, out| {
            if let StaleOutcome::Recompute { .. } = out {
                requeue.push(k.1);
            }
        });
        for pid in requeue {
            if let Some(l) = live.remove(&pid) {
                m.stale_recomputes += 1;
                finder.note_stale_recompute();
                pending.push(Req {
                    id: pid,
                    start: l.start,
                    goal: l.goal,
                });
            }
        }

        // (2) 事务交错 churn（单向删边口径）
        if tick % TX_EVERY == 0 {
            let mut ops = Vec::new();
            for k in 0..TX_REMOVE {
                let eid = event_u64(SEED, "tx", tick, k) % n_edges;
                ops.push(Op::RemoveEdge { id: eid });
            }
            if net.publish(Tx { id: tick, ops }).is_ok() {
                m.tx_published += 1;
            } else {
                m.tx_rejected += 1;
            }
        }

        // (3) 串行权威：请求 ID 稳定序 + TickBudget
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
                FindOutcome::Unreachable => {
                    m.blocked_unreachable += 1;
                }
            }
        }
        pending = still;

        // (4) 逐 tick 折叠摘要
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
    m.wall_ms = t0.elapsed().as_secs_f64() * 1000.0;
    (gd.finalize().hex(), m)
}

fn main() {
    println!("== Stage 2B 并发队列压测（200x200, {REQUESTS} 请求, churn 每 {TX_EVERY} tick 删 {TX_REMOVE} 边, Q={QUOTA}/tick）==");
    println!("口径：budget_yield_count=累计让步事件；stale_recomputes=路网轴重算（两轴分离）；recompute 分母=stale 查询总数");
    let mut digests = Vec::new();
    for (threads, chaos) in [(1usize, 0u64), (2, 0), (8, 0), (4, 42)] {
        let (d, m) = run(threads, chaos);
        println!(
            "threads={threads} chaos={chaos}: digest={} ticks={} completed={} unreachable={} yields={} recomputes={} tx_ok={} tx_rej={} sweeps={} pending_left={} wall={:.1}ms",
            &d[..16], m.ticks, m.completed, m.blocked_unreachable, m.yield_events,
            m.stale_recomputes, m.tx_published, m.tx_rejected, m.stale_sweeps, m.pending_left, m.wall_ms
        );
        println!("  full digest: {d}");
        digests.push(d);
        assert_eq!(m.pending_left, 0, "全部请求必须终态");
        assert_eq!(
            m.completed + m.blocked_unreachable,
            REQUESTS + m.stale_recomputes,
            "会计恒等式"
        );
    }
    let same = digests.windows(2).all(|w| w[0] == w[1]);
    println!(
        "线程无关（1/2/8）+混沌无关（4）摘要一致：{}",
        if same { "PASS" } else { "FAIL" }
    );
    assert!(same, "摘要必须一致");
}
