//! 专用耦合线程无关测试（ADR-NEW-25 3c；Stage 2B chaos 口径）。
//!
//! 覆盖"MFD→降级信号"完整生产路径：
//!   车辆快照 →（sim-parallel 并行聚合，含 chaos_spin 忙等注入延迟）→ 需求压力
//!   demand_pressure_bp（确定性定点）→ BudgetRegistry::note_demand_pressure
//!   （权威降级状态机）→ end_tick → GlobalDigest。
//!
//! 判据：同一场景在 1/2/8 线程与 chaos_seed 0/42 下摘要逐一相等，且与纯串行
//! 参考实现一致——通用并行证据不替代本耦合验证（决策2 投票要求）。

use sim_codec::CanonicalWriter;
use sim_core::budget::{
    BudgetDecl, BudgetRegistry, DegradationThresholds, OverflowBehavior, WorkUnit,
};
use sim_digest::GlobalDigest;
use sim_parallel::{run_parallel, StableKey, Task};
use sim_traffic::GridTraffic;

const W: u32 = 16;
const H: u32 = 16;
const SEED: u64 = 0x00C0_FFEE_1234_5678;
const K_REF_PERMILLE: u64 = 500; // 需求压力分母（特征密度 k_ref，ADR-NEW-26 决策4 修订；占位值，待凹形模型定案回填）
const TICKS: u64 = 30;
const CHUNK: usize = 4;

type Snap = Vec<(u64, u32, u32)>; // (id, x, y) 快照

fn count_chunk(s: &Snap, in_: &(usize, usize, u64)) -> u64 {
    // 只读快照计数；chaos_spin 忙等注入延迟（确定性，不改变输出）
    let mut n = 0u64;
    for v in &s[in_.0..in_.1] {
        sim_parallel_chaos::spin(v.0, in_.2);
        std::hint::black_box(v);
        n += 1;
    }
    n
}

// chaos_spin 由 sim-test-support 提供；本地薄封装避免改动 sim-parallel 公开面。
mod sim_parallel_chaos {
    pub fn spin(task_id: u64, chaos_seed: u64) {
        // 与 sim-test-support::chaos::chaos_spin 相同算法（splitmix64 finalizer % 1024 次忙等）
        let mut x = task_id ^ chaos_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut n = 0u64;
        if chaos_seed != 0 {
            x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            x ^= x >> 31;
            n = x % 1024;
        }
        let mut c = 0u64;
        while c < n {
            std::hint::black_box(c);
            c += 1;
        }
    }
}

fn run_scenario(threads: usize, chaos_seed: u64, parallel: bool) -> [u8; 32] {
    let mut gt = GridTraffic::new(W, H, 1, SEED);
    let mut reg = BudgetRegistry::new(
        vec![BudgetDecl {
            system_id: 1,
            unit: WorkUnit::EntityUpdates,
            per_tick_cap: 1000,
            overflow: OverflowBehavior::ShardYield,
        }],
        DegradationThresholds::default(),
    );
    // 决策6 Z2 护栏：需求通道默认冻结——耦合测试显式解冻（本测试即 ADR-NEW-26
    // 通道的专用耦合证据；生产代码须待 ADR-NEW-26 解冻裁决后才允许调用）。
    reg.unfreeze_demand_channel();
    let mut g = GlobalDigest::new();
    for t in 0..TICKS {
        gt.inject(2 + t % 3);
        gt.step();
        // 快照（权威状态只读视图）
        let snap: Snap = gt
            .vehicles
            .iter()
            .map(|v| (v.id, v.x, v.y))
            .collect();
        let count = if parallel {
            let mut tasks = Vec::new();
            for (ci, r) in (0..snap.len()).step_by(CHUNK).enumerate() {
                let end = (r + CHUNK).min(snap.len());
                tasks.push(Task {
                    key: (1u32, ci as u64),
                    input: (r, end, chaos_seed),
                    func: count_chunk,
                });
            }
            let mut outs: Vec<(StableKey, u64)> = Vec::new();
            run_parallel(&snap, tasks, threads, |k, out| outs.push((k, out)));
            outs.sort_by_key(|(k, _)| *k);
            outs.iter().map(|(_, o)| *o).sum::<u64>()
        } else {
            (0..snap.len()).step_by(CHUNK).map(|r| {
                let end = (r + CHUNK).min(snap.len());
                count_chunk(&snap, &(r, end, chaos_seed))
            }).sum::<u64>()
        };
        // 需求压力通道（确定性定点：计数→密度→压力）
        // 决策4 整改：并行聚合结果必须参与被测信号（原 let _ = count 被指未参与）。
        assert_eq!(count, gt.active_count(), "parallel aggregation must match authoritative state");
        let pressure = gt.demand_pressure_bp(K_REF_PERMILLE);
        reg.note_demand_pressure(pressure);
        reg.spend(1, t % 50); // 混合工作量通道
        reg.end_tick();
        gt.fold_digest(&mut g);
        let mut w = CanonicalWriter::new("sim-core", "BudgetState");
        reg.encode_state(&mut w);
        g.record("sim-core", "BudgetState", w.as_bytes());
    }
    g.finalize().0
}

#[test]
fn mfd_degradation_coupling_is_thread_independent() {
    let r1 = run_scenario(1, 0, true);
    let r2 = run_scenario(2, 0, true);
    let r8 = run_scenario(8, 0, true);
    assert_eq!(r1, r2, "1 vs 2 threads");
    assert_eq!(r2, r8, "2 vs 8 threads");
}

#[test]
fn mfd_degradation_coupling_is_chaos_independent() {
    let a = run_scenario(4, 0, true);
    let b = run_scenario(4, 42, true);
    assert_eq!(a, b, "chaos seed must not affect the digest");
}

#[test]
fn mfd_degradation_coupling_matches_serial_reference() {
    let par = run_scenario(4, 42, true);
    let ser = run_scenario(1, 0, false);
    assert_eq!(par, ser, "parallel pipeline must match serial reference");
}
