//! Set D 复测（债务 #31 核销条件；ADR-NEW-25 3a 决策4 判据修订 (c)）。
//!
//! 预注册 fixture（**运行前声明，先于 Set D 数据**）：
//! - Set D 种子段=0x00C0_FFEE_2222_0000 + i（i=0..8），与 V1 的 A/B/C 段
//!   （0x00C0_FFEE_1234_5678+i）不相交；
//! - 拓扑：主 40×40 + 第二拓扑 64×64（判据修订要求）；
//! - 密度阶梯：拟合集 2→256 倍增 8 档；留出集 3→384 倍增 8 档（交错不相交）；
//! - 每档 260 tick，末 60 tick 窗口均值；稳态判据=窗口前后半 q 差 ≤2‰；
//! - 每档 8 种子平均；分区边界 k_turn=k_half（拟合自拟合集，先于留出评估声明）；
//! - 判据（常量单一来源=sim-traffic）：自由流 ≤100‰、转折 ≤250‰、饱和 ≤250‰。
//!
//! 终止条件（Claude）：任一拓扑任一区 FAIL → 转 C，不再延后。
//!
//! 运行：cargo run --release --example mfd_setd -p sim-traffic

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use sim_traffic::{
    evaluate_rational_fit, fit_rational_saturation_detailed, q_rational_permille, GridTraffic,
    MfdSample, FREE_FLOW_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE,
};

const SETD_SEED_BASE: u64 = 0x00C0_FFEE_2222_0000;
const N_SEEDS: u64 = 8;
const TICKS: usize = 260;
const WINDOW: usize = 60;
const FIT_LADDER: [u64; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
const HOLDOUT_LADDER: [u64; 8] = [3, 6, 12, 24, 48, 96, 192, 384];

/// 单运行：返回 (k̄, q̄) 窗口均值与稳态判定。
fn run_density_seed(w: u32, h: u32, inj: u64, seed: u64) -> (u64, u64, bool) {
    let mut gt = GridTraffic::new(w, h, 1, seed);
    let mut k_sum = 0u64;
    let mut q_sum = 0u64;
    let mut q_front = 0u64;
    let mut q_back = 0u64;
    for t in 0..TICKS {
        gt.inject(inj);
        gt.step();
        if t >= TICKS - WINDOW {
            k_sum += gt.active_count() * 1000 / gt.edge_count();
            q_sum += gt.flow_this_tick * 1000 / gt.edge_count();
            if t >= TICKS - WINDOW / 2 {
                q_back += gt.flow_this_tick;
            } else {
                q_front += gt.flow_this_tick;
            }
        }
    }
    let n = WINDOW as u64;
    // 稳态判据双口径：相对差 ≤2‰，或绝对差 ≤ 半窗口一半（≈0.5 车/tick）——
    // 小流量点 1 辆车波动即超 2‰，纯相对口径是测量分辨率问题非发散（W2 实测教训）。
    let steady = q_front.abs_diff(q_back) * 1000 <= (q_front + q_back) * 2
        || q_front.abs_diff(q_back) <= (WINDOW as u64) / 4;
    (k_sum / n, q_sum / n, steady)
}

/// 多种子平均采样。稳态仅作诊断（不阻断）：返回 (样本, 未稳态档数)。
fn sample_ladder(w: u32, h: u32, ladder: &[u64]) -> (Vec<MfdSample>, usize) {
    let mut unsteady = 0usize;
    let samples = ladder
        .iter()
        .map(|&inj| {
            let (mut k, mut q, mut steady) = (0u64, 0u64, true);
            for i in 0..N_SEEDS {
                let (k1, q1, s1) = run_density_seed(w, h, inj, SETD_SEED_BASE + i);
                k += k1;
                q += q1;
                steady &= s1;
            }
            if !steady {
                unsteady += 1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    (samples, unsteady)
}

fn report_topology(w: u32, h: u32, emit: &mut dyn FnMut(String)) {
    emit(format!("=== topology {w}x{h} (Set D seeds {SETD_SEED_BASE:#x}+0..{N_SEEDS}) ==="));
    let (mut fit_samples, unsteady_fit) = sample_ladder(w, h, &FIT_LADDER);
    fit_samples.sort_by_key(|s| s.k_permille);
    let (holdout, unsteady_hold) = sample_ladder(w, h, &HOLDOUT_LADDER);
    emit(format!(
        "steady diagnostics: fit {}/{} steady, holdout {}/{} steady (dual criterion; unsteady ladders disclosed)",
        FIT_LADDER.len() - unsteady_fit,
        FIT_LADDER.len(),
        HOLDOUT_LADDER.len() - unsteady_hold,
        HOLDOUT_LADDER.len()
    ));
    let (fit, free_slope, binding) = fit_rational_saturation_detailed(&fit_samples);
    emit(format!(
        "fit: q_max={}‰ k_half={}‰ zero-density slope={}‰ (free estimate {}‰, bound binding: {})",
        fit.q_max_permille,
        fit.k_half_permille,
        fit.zero_density_slope_permille(),
        free_slope,
        binding
    ));
    for s in &holdout {
        let qm = q_rational_permille(&fit, s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000 / (s.q_permille.max(1) as u128);
        emit(format!(
            "  k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ err {}‰",
            s.k_permille, s.q_permille, qm, err
        ));
    }
    let r = evaluate_rational_fit(&fit, &holdout);
    let verdict = |v: u64, c: u64| if v <= c { "PASS" } else { "FAIL" };
    emit(format!(
        "  free-flow max err   = {:>4}‰  (criterion <= {}‰)  {}",
        r.free_flow_max_err_permille, FREE_FLOW_CRIT_PERMILLE,
        verdict(r.free_flow_max_err_permille, FREE_FLOW_CRIT_PERMILLE)
    ));
    emit(format!(
        "  transition max err  = {:>4}‰  (criterion <= {}‰)  {}",
        r.transition_max_err_permille, TRANSITION_CRIT_PERMILLE,
        verdict(r.transition_max_err_permille, TRANSITION_CRIT_PERMILLE)
    ));
    emit(format!(
        "  saturated max err   = {:>4}‰  (criterion <= {}‰)  {}",
        r.saturated_max_err_permille, SATURATED_CRIT_PERMILLE,
        verdict(r.saturated_max_err_permille, SATURATED_CRIT_PERMILLE)
    ));
}

fn main() {
    let mut out: Vec<String> = Vec::new();
    out.push("Set D re-verification (debt #31; pre-registered fixture, disjoint seeds)".to_string());
    out.push(format!(
        "criteria: free<={}‰ transition<={}‰ saturated<={}‰ (single source = sim-traffic consts)",
        FREE_FLOW_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE
    ));
    report_topology(40, 40, &mut |l| out.push(l));
    report_topology(64, 64, &mut |l| out.push(l));
    for l in &out {
        println!("{l}");
    }
}
