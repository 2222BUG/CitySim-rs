//! R2 确定性场景骨架（Stage 1 T9）：sim-test-support 的 SimHarness 适配 sim-save。
//!
//! 目的：把 EC-11 R2 的三种确定性检查（同种子复现、强读档等价、固定场景）
//! 接到正式存档/模拟状态上，作为 R2 门禁（CI r2-deterministic 占位 job）的
//! 启用前置。完整 R2（10,000 Agent × 1,000 Tick）待模拟核心 tick 循环完备后启用。
//!
//! 摘要口径：SimHarness 接口以 u64 摘要交互；此处将 BLAKE3-256 摘要折叠为 u64
//! 仅作接口适配——R2 oracle 的正式比对使用 sim_save::state_digest 完整 256 位值。

use sim_save::{state_digest, SimState};
use sim_test_support::deterministic::{
    repeat_run_digests, strong_readback_digests, weak_form_digests, SimHarness,
};
use sim_test_support::fixtures::Fixture;

struct Harness {
    state: SimState,
}

impl Harness {
    /// 骨架口径：agent 数以 fixture 为准但封顶 100（R2 全量 10k 待核心完备）。
    fn from_fixture(f: &Fixture) -> Self {
        let mut s = SimState::new(1, f.seed);
        for i in 0..f.agent_count.min(100) {
            s.spawn_agent(i as i64 * 1000, i as i64 * 2000 + 500);
        }
        Harness { state: s }
    }
}

fn fold_u64(d: &sim_save::Digest256) -> u64 {
    let b = d.0;
    let mut v = 0u64;
    for chunk in b.chunks_exact(8) {
        v ^= u64::from_le_bytes(chunk.try_into().unwrap());
    }
    v
}

impl SimHarness for Harness {
    fn run_to(&mut self, tick: u64) -> u64 {
        while self.state.tick < tick {
            self.state.run_tick();
        }
        fold_u64(&state_digest(&self.state))
    }

    fn save(&self) -> Vec<u8> {
        self.state.save().expect("save at tick boundary").0
    }

    fn load(&mut self, blob: &[u8]) {
        self.state = SimState::load(blob).expect("load must pass consistency check");
    }
}

/// 退出条件 1：同一种子运行两次，摘要一致（可复现模拟循环）。
#[test]
fn r2_same_seed_two_runs_agree() {
    let f = Fixture::smoke();
    let (d1, d2) = repeat_run_digests(|| Harness::from_fixture(&f), 100);
    assert_eq!(d1, d2, "same seed must reproduce identical digest");
}

/// 退出条件 2：强读档等价经 sim-test-support 模板（N=40 存档读档跑至 M=100）。
#[test]
fn r2_strong_readback_via_harness() {
    let f = Fixture::smoke();
    let (d_direct, d_readback) = strong_readback_digests(|| Harness::from_fixture(&f), 40, 100);
    assert_eq!(
        d_direct, d_readback,
        "strong readback equivalence must hold through the harness"
    );
}

/// 弱形式回归（政策口径变更时的降级路径可用）。
#[test]
fn r2_weak_form_available() {
    let f = Fixture::smoke();
    let (d1, d2) = weak_form_digests(|| Harness::from_fixture(&f), 40, 100);
    assert_eq!(d1, d2);
}

/// 固定场景可跑（fixture 驱动，R2 全量档位待核心完备后切 r2_gate()）。
#[test]
fn r2_fixture_smoke_runs_deterministically() {
    let f = Fixture::smoke();
    let mut h = Harness::from_fixture(&f);
    let d1 = h.run_to(50);
    let mut h2 = Harness::from_fixture(&f);
    let d2 = h2.run_to(50);
    assert_eq!(d1, d2);
    assert_eq!(f.tick_count > 0, true, "fixture defines a tick budget");
}
