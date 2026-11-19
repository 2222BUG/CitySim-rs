//! B3 场景级线程无关性验证（Stage 2B 退出条件 1/2/3/4）。
//!
//! 场景：100 个 Agent × 50 tick 的确定性随机游走。
//! 每 tick：只读快照上并行执行纯函数任务（含混沌忙等注入），
//! 按稳定键排序串行提交，逐 tick 折叠 GlobalDigest。
//! 断言：1/2/8 线程摘要一致、混沌种子无关、任务到达顺序无关、与单线程参考一致。

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_parallel::{run_parallel, StableKey, Task};
use sim_rng::event_u64;
use sim_test_support::chaos::chaos_spin;

const SEED: u64 = 0x1234_5678_9ABC_DEF0;
const AGENTS: u64 = 100;
const TICKS: u64 = 50;

#[derive(Clone)]
struct World {
    agents: Vec<(i64, i64)>, // 索引 = agent_id - 1
}

type In = (u64, u64, u64); // (agent_id, tick, chaos_seed)
type Out = (i64, i64);

/// 纯函数任务：只读快照 + 确定性派生位移 + 混沌忙等（只影响耗时，不影响结果）。
fn step_task(snap: &World, input: &In) -> Out {
    let (aid, tick, chaos_seed) = *input;
    let _ = chaos_spin(aid.saturating_mul(tick), chaos_seed);
    let (x, y) = snap.agents[(aid - 1) as usize];
    let r = event_u64(
        SEED,
        "sim-parallel",
        aid.saturating_mul(1000).saturating_add(tick),
        0,
    );
    let dx = (r % 5) as i64 - 2;
    let dy = ((r >> 8) % 5) as i64 - 2;
    (x.saturating_add(dx), y.saturating_add(dy))
}

fn record_tick(gd: &mut GlobalDigest, tick: u64, world: &World) {
    let mut w = CanonicalWriter::new("sim-parallel", "AgentState");
    w.u64(tick);
    for p in &world.agents {
        w.i64(p.0);
        w.i64(p.1);
    }
    gd.record("sim-parallel", "AgentState", w.as_bytes());
}

/// 场景执行：并行框架路径。reversed=true 时任务构建顺序颠倒（检验提交契约）。
fn run_scenario(threads: usize, chaos_seed: u64, reversed: bool) -> String {
    let mut world = World {
        agents: (1..=AGENTS).map(|i| (i as i64 * 7, i as i64 * 11)).collect(),
    };
    let mut gd = GlobalDigest::new();
    for tick in 1..=TICKS {
        let snap = world.clone();
        let mut tasks: Vec<Task<World, In, Out>> = (1..=AGENTS)
            .map(|aid| Task {
                key: (0, aid),
                input: (aid, tick, chaos_seed),
                func: step_task,
            })
            .collect();
        if reversed {
            tasks.reverse();
        }
        run_parallel(&snap, tasks, threads, |k, out| {
            // 串行提交（run_parallel 已按稳定键排序逐项回调）
            world.agents[(k.1 - 1) as usize] = out;
        });
        record_tick(&mut gd, tick, &world);
    }
    gd.finalize().hex()
}

/// 单线程参考实现：同样的纯函数与排序提交，完全不走并行框架。
fn run_serial(chaos_seed: u64) -> String {
    let mut world = World {
        agents: (1..=AGENTS).map(|i| (i as i64 * 7, i as i64 * 11)).collect(),
    };
    let mut gd = GlobalDigest::new();
    for tick in 1..=TICKS {
        let snap = world.clone();
        let tasks: Vec<Task<World, In, Out>> = (1..=AGENTS)
            .map(|aid| Task {
                key: (0, aid),
                input: (aid, tick, chaos_seed),
                func: step_task,
            })
            .collect();
        let mut results: Vec<(StableKey, Out)> = tasks
            .iter()
            .map(|t| (t.key, (t.func)(&snap, &t.input)))
            .collect();
        results.sort_by_key(|(k, _)| *k);
        for (k, out) in results {
            world.agents[(k.1 - 1) as usize] = out;
        }
        record_tick(&mut gd, tick, &world);
    }
    gd.finalize().hex()
}

#[test]
fn thread_count_independence_1_2_8() {
    let d1 = run_scenario(1, 0, false);
    let d2 = run_scenario(2, 0, false);
    let d8 = run_scenario(8, 0, false);
    assert_eq!(d1, d2, "1 线程与 2 线程摘要必须一致");
    assert_eq!(d1, d8, "1 线程与 8 线程摘要必须一致");
}

#[test]
fn chaos_injection_does_not_change_result() {
    let d0 = run_scenario(4, 0, false);
    let d42 = run_scenario(4, 42, false);
    assert_eq!(d0, d42, "混沌注入只影响耗时，不得影响结果");
}

#[test]
fn submission_order_independence_scenario() {
    let forward = run_scenario(2, 0, false);
    let reversed = run_scenario(2, 0, true);
    assert_eq!(forward, reversed, "任务到达顺序不得影响提交结果");
}

#[test]
fn parallel_matches_serial_reference() {
    let par = run_scenario(4, 0, false);
    let ser = run_serial(0);
    assert_eq!(par, ser, "并行与单线程参考结果必须一致");
}
