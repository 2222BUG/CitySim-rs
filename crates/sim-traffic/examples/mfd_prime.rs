//! ②′ 确认轮（债务 #31 修测一轮定生死；ADR-NEW-25"②′ 确认轮预注册规格"
//! 2026-11-22 决策6 Z3 预注册于 7160955，本文件=fixture 实装，先落盘后取数）。
//!
//! 预注册 fixture（**运行前声明，先于 ②′ 数据**）：
//! - 种子段=0x00C0_FFEE_4444_0000 + i，三段独立：fit +0..8 / holdout +8..16 /
//!   back-half fit +16..24、back-half holdout +24..32（与 V1/Set D/Set E 段不相交）；
//! - 拓扑：40×40 + 32×32 + 64×64（三拓扑全跑）；
//! - 密度阶梯**低密几何加密**（②′ 修测：自由流区判定点 n≥5，抗单点杠杆）：
//!   拟合集 18 档、留出集 18 档（×~1.4 交错）；
//! - 每档 260 tick，末 60 tick 窗口均值；稳态判据=sim-traffic::steady_window
//!   双口径（≤2‰ 或 ≤15 辆，文档与实现统一）；非稳态档单独报告不静默；
//! - 每档 8 种子平均；分区边界=ZoneBands::SET_D 预注册绝对带（k<100 / 100..2000 /
//!   ≥2000‰）；判据（常量单一来源=sim-traffic）：自由流 ≤100‰、转折 ≤250‰、
//!   饱和 ≤250‰；每点分辨率下限=point_resolution_ok，不满足标"分辨率受限"
//!   并以绝对误差 |Δq|≤2‰ 复核披露；
//! - 族集冻结={rational, pnorm-p2}（扩族须重新投票——决策5 共识）；
//! - 预选（Q3）=物理可行域内**判据归一化损失**（criterion_aligned_loss）
//!   最小者一次锁定，不得回退；全 binding→无族可选=FAIL。
//!
//! **终止条款（焊死）**：任一拓扑任一区 FAIL → 自动落 ①（追认转 C，MFD 永久纯
//! 统计），无须再投票；全部 PASS → #31 核销、3a 记通过（解冻另走 ADR-NEW-26）。
//!
//! 运行：SETD_COMMIT=$(git rev-parse HEAD) cargo run --release --example mfd_prime -p sim-traffic

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use sim_traffic::{
    criterion_aligned_loss, evaluate_pnorm_fit, evaluate_rational_fit,
    fit_pnorm_concave_detailed, fit_rational_saturation_detailed, limited_cap_ok,
    point_resolution_ok,
    q_pnorm_permille, q_rational_permille, steady_window, FitErrorReport, GridTraffic, MfdSample,
    RationalSaturationFit, ZoneBands, FREE_FLOW_CRIT_PERMILLE,
    SATURATED_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE,
};

const SEED_BASE: u64 = 0x00C0_FFEE_4444_0000;
const N_SEEDS: u64 = 8;
const TICKS: usize = 260;
const WINDOW: usize = 60;
/// 低密几何加密（②′ 修测：自由流区 n≥5）。
const FIT_LADDER: [u64; 18] =
    [1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512];
/// 留出集阶梯（×~1.4 交错，与拟合集同密度范围覆盖三区）。
const HOLDOUT_LADDER: [u64; 18] =
    [2, 3, 5, 7, 9, 13, 18, 26, 36, 52, 72, 104, 144, 208, 288, 416, 576, 768];

fn zone_crit(k_permille: u64) -> u64 {
    if k_permille < 100 {
        FREE_FLOW_CRIT_PERMILLE
    } else if k_permille < 2000 {
        TRANSITION_CRIT_PERMILLE
    } else {
        SATURATED_CRIT_PERMILLE
    }
}

/// 单运行：返回 (k̄, q̄) 窗口均值与稳态判定（steady_window 双口径统一）。
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
            assert!(
                gt.flow_this_tick <= gt.last_step_population,
                "q>k impossible under same-population accounting"
            );
            k_sum += gt.last_step_population * 1000 / gt.edge_count();
            q_sum += gt.flow_this_tick * 1000 / gt.edge_count();
            if t >= TICKS - WINDOW / 2 {
                q_back += gt.flow_this_tick;
            } else {
                q_front += gt.flow_this_tick;
            }
        }
    }
    let n = WINDOW as u64;
    let delta = q_front.abs_diff(q_back);
    let delta_permille = delta * 1000 / (q_front + q_back).max(1);
    (k_sum / n, q_sum / n, steady_window(delta_permille, delta))
}

/// 窗口后半采样（back-half 敏感性）。
fn run_density_seed_back_half(w: u32, h: u32, inj: u64, seed: u64) -> (u64, u64) {
    let mut gt = GridTraffic::new(w, h, 1, seed);
    let mut k_sum = 0u64;
    let mut q_sum = 0u64;
    for t in 0..TICKS {
        gt.inject(inj);
        gt.step();
        if t >= TICKS - WINDOW / 2 {
            k_sum += gt.last_step_population * 1000 / gt.edge_count();
            q_sum += gt.flow_this_tick * 1000 / gt.edge_count();
        }
    }
    let n = (WINDOW / 2) as u64;
    (k_sum / n, q_sum / n)
}

/// 多种子平均采样。返回 (样本, 逐档稳态标记)。
fn sample_ladder(w: u32, h: u32, ladder: &[u64], seed_off: u64) -> (Vec<MfdSample>, Vec<bool>) {
    let mut flags = Vec::new();
    let samples = ladder
        .iter()
        .map(|&inj| {
            let (mut k, mut q, mut steady) = (0u64, 0u64, true);
            for i in 0..N_SEEDS {
                let (k1, q1, s1) = run_density_seed(w, h, inj, SEED_BASE + seed_off + i);
                k += k1;
                q += q1;
                steady &= s1;
            }
            flags.push(steady);
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    (samples, flags)
}

type QModelFn<'a> = Box<dyn Fn(u64) -> u64 + 'a>;

fn rel_sse(q: impl Fn(u64) -> u64, samples: &[MfdSample]) -> u128 {
    let mut sse = 0u128;
    for s in samples {
        let e = q(s.k_permille) as i128 - s.q_permille as i128;
        let er = (e * 1000) / (s.q_permille.max(1) as i128);
        sse += (er * er) as u128;
    }
    sse
}

/// 结构化判定核（审计3 C-1/C-2：判定由数据字段驱动，展示文本只是投影）。
/// PASS 条件：非 binding（决策4 条件⑧）+ 三区逐点判定（含分辨率双分支复核，
/// 经 FitErrorReport.free_pass/trans_pass/sat_pass）+ 分辨率受限占比 ≤1/18
/// （豆包 R2 上限）。空区 pass 字段已为 false。
fn family_pass(binding: bool, r: &FitErrorReport) -> bool {
    !binding
        && r.free_pass
        && r.trans_pass
        && r.sat_pass
        && limited_cap_ok(
            r.free_n_limited + r.trans_n_limited + r.sat_n_limited,
            r.points_evaluated,
        )
}

fn family_verdict(binding: bool, r: &FitErrorReport) -> &'static str {
    if family_pass(binding, r) {
        "PASS"
    } else {
        "FAIL"
    }
}

/// 归因对照专用（P3，非判定路径）：旧 rel-SSE 目标网格辨识（明确标注控制组）。
fn fit_rational_rel_sse_control(samples: &[MfdSample]) -> RationalSaturationFit {
    let q_obs_max = samples.iter().map(|s| s.q_permille).max().unwrap().max(1);
    let k_max = samples.iter().map(|s| s.k_permille).max().unwrap().max(1);
    let mut best: Option<(u128, u64, u64)> = None;
    for q_max in q_obs_max..=q_obs_max.saturating_mul(4).saturating_add(16) {
        for k_half in 1..=k_max.saturating_mul(4).max(1) {
            let fit = RationalSaturationFit { q_max_permille: q_max, k_half_permille: k_half };
            let sse = rel_sse(|k| q_rational_permille(&fit, k), samples);
            if best.is_none_or(|(b, _, _)| sse < b) {
                best = Some((sse, q_max, k_half));
            }
        }
    }
    let (_, q, k) = best.unwrap();
    RationalSaturationFit { q_max_permille: q, k_half_permille: k }
}

/// 逐点行后缀：分辨率标注（②′ 修测）。
fn res_tag(k: u64, q_obs: u64, q_model: u64) -> String {
    if point_resolution_ok(q_obs, zone_crit(k)) {
        "res=ok".to_string()
    } else {
        format!(
            "res=LIMITED(abs_err={}‰, |Δq|≤2‰ 复核{})",
            q_model.abs_diff(q_obs),
            if q_model.abs_diff(q_obs) <= 2 { ":PASS" } else { ":FAIL" }
        )
    }
}

fn report_topology(w: u32, h: u32, out: &mut Vec<String>) -> bool {
    let mut emit = |l: String| out.push(l);
    emit(format!("=== topology {w}x{h} (②′ seeds {SEED_BASE:#x}+0..{N_SEEDS}) ==="));
    let (mut fit_samples, fit_flags) = sample_ladder(w, h, &FIT_LADDER, 0);
    fit_samples.sort_by_key(|s| s.k_permille);
    let (holdout, hold_flags) = sample_ladder(w, h, &HOLDOUT_LADDER, 8);
    let steady_fit = fit_flags.iter().filter(|&&s| s).count();
    let steady_hold = hold_flags.iter().filter(|&&s| s).count();
    emit(format!(
        "steady: fit {}/{} holdout {}/{} (dual criterion steady_window; unsteady ladders: fit {:?} holdout {:?})",
        steady_fit,
        FIT_LADDER.len(),
        steady_hold,
        HOLDOUT_LADDER.len(),
        FIT_LADDER
            .iter()
            .zip(&fit_flags)
            .filter(|(_, s)| !**s)
            .map(|(i, _)| *i)
            .collect::<Vec<_>>(),
        HOLDOUT_LADDER
            .iter()
            .zip(&hold_flags)
            .filter(|(_, s)| !**s)
            .map(|(i, _)| *i)
            .collect::<Vec<_>>(),
    ));

    // ===== 辨识两族（判据归一化损失目标，P1 修测）=====
    let (fit, free_slope, binding) =
        fit_rational_saturation_detailed(&fit_samples, &ZoneBands::SET_D);
    let (pfit, p_slope, p_binding) =
        fit_pnorm_concave_detailed(&fit_samples, &ZoneBands::SET_D);
    emit(format!(
        "fit rational: q_max={}‰ k_half={}‰ slope={}‰ (free est {}‰ binding={})",
        fit.q_max_permille,
        fit.k_half_permille,
        fit.zero_density_slope_permille(),
        free_slope,
        binding
    ));
    emit(format!(
        "fit pnorm-p2: q_max={}‰ k0={}‰ slope={}‰ (free est {}‰ binding={})",
        pfit.q_max_permille,
        pfit.k0_permille,
        pfit.zero_density_slope_permille(),
        p_slope,
        p_binding
    ));

    // ===== Q3 预选（判据归一化损失，一次锁定；fit-set 逐点归档）=====
    let qf_tab: Vec<(&str, QModelFn<'_>)> = vec![
        ("rational", Box::new(|k: u64| q_rational_permille(&fit, k))),
        ("pnorm-p2", Box::new(|k: u64| q_pnorm_permille(&pfit, k))),
    ];
    for (tag, qf) in &qf_tab {
        for s in &fit_samples {
            let qm = qf(s.k_permille);
            emit(format!(
                "  [fit-set {tag}] k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ {}",
                s.k_permille,
                s.q_permille,
                qm,
                res_tag(s.k_permille, s.q_permille, qm)
            ));
        }
    }
    let loss_r =
        criterion_aligned_loss(|k| q_rational_permille(&fit, k), &fit_samples, &ZoneBands::SET_D);
    let loss_p =
        criterion_aligned_loss(|k| q_pnorm_permille(&pfit, k), &fit_samples, &ZoneBands::SET_D);
    let sse_r = rel_sse(|k| q_rational_permille(&fit, k), &fit_samples);
    let sse_p = rel_sse(|k| q_pnorm_permille(&pfit, k), &fit_samples);
    let selected: Option<&str> = match (binding, p_binding) {
        (true, true) => None,
        (false, true) => Some("rational"),
        (true, false) => Some("pnorm-p2"),
        (false, false) => Some(if loss_p < loss_r { "pnorm-p2" } else { "rational" }),
    };
    emit(format!(
        "  [family pre-selection] criterion-loss rational={loss_r} pnorm={loss_p} (rel-SSE disclosure r={sse_r} p={sse_p}) binding(r={binding},p={p_binding}) -> selected={} (Q3 feasible-domain criterion-loss rule; locked before holdout)",
        selected.unwrap_or("NONE (all binding)")
    ));

    // ===== 留出集逐点表（双族并列+分辨率标注）=====
    for s in &holdout {
        let q_r = q_rational_permille(&fit, s.k_permille);
        let q_p = q_pnorm_permille(&pfit, s.k_permille);
        emit(format!(
            "  k {:>5}‰ q_obs {:>4}‰ | rational q {:>4}‰ {} | pnorm-p2 q {:>4}‰ {}",
            s.k_permille,
            s.q_permille,
            q_r,
            res_tag(s.k_permille, s.q_permille, q_r),
            q_p,
            res_tag(s.k_permille, s.q_permille, q_p)
        ));
    }

    let r = evaluate_rational_fit(&fit, &holdout, &ZoneBands::SET_D);
    let rp = evaluate_pnorm_fit(&pfit, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "  [rational] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        r.free_flow_max_err_permille,
        r.free_n,
        r.transition_max_err_permille,
        r.trans_n,
        r.saturated_max_err_permille,
        r.sat_n,
        binding,
        family_verdict(binding, &r)
    ));
    emit(format!(
        "  [pnorm-p2] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        rp.free_flow_max_err_permille,
        rp.free_n,
        rp.transition_max_err_permille,
        rp.trans_n,
        rp.saturated_max_err_permille,
        rp.sat_n,
        p_binding,
        family_verdict(p_binding, &rp)
    ));
    // 结构化裁决（审计3 C-2：布尔值是源，文本是投影——自动终止不得由文本驱动）。
    let outcome_pass = match selected {
        Some("pnorm-p2") => family_pass(p_binding, &rp),
        Some(_) => family_pass(binding, &r),
        None => false,
    };
    emit(format!(
        "  resolution-limited points (holdout): free={} trans={} sat={} cap_ok={}",
        r.free_n_limited,
        r.trans_n_limited,
        r.sat_n_limited,
        limited_cap_ok(
            r.free_n_limited + r.trans_n_limited + r.sat_n_limited,
            r.points_evaluated
        )
    ));
    emit(format!(
        "  Prime outcome (selected family {}) = {}",
        selected.unwrap_or("NONE"),
        if outcome_pass { "PASS" } else { "FAIL" }
    ));

    // ===== back-half 敏感性（独立种子段 +16..24 / +24..32）=====
    let back_fit: Vec<MfdSample> = FIT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SEED_BASE + 16 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let back_hold: Vec<MfdSample> = HOLDOUT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SEED_BASE + 24 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let (fit_b, _, binding_b) = fit_rational_saturation_detailed(&back_fit, &ZoneBands::SET_D);
    let r_b = evaluate_rational_fit(&fit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [back-half rational] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        r_b.free_flow_max_err_permille,
        r_b.free_n,
        r_b.transition_max_err_permille,
        r_b.trans_n,
        r_b.saturated_max_err_permille,
        r_b.sat_n,
        binding_b,
        family_verdict(binding_b, &r_b)
    ));
    let (pfit_b, _, p_binding_b) = fit_pnorm_concave_detailed(&back_fit, &ZoneBands::SET_D);
    let rp_b = evaluate_pnorm_fit(&pfit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [back-half pnorm-p2] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        rp_b.free_flow_max_err_permille,
        rp_b.free_n,
        rp_b.transition_max_err_permille,
        rp_b.trans_n,
        rp_b.saturated_max_err_permille,
        rp_b.sat_n,
        p_binding_b,
        family_verdict(p_binding_b, &rp_b)
    ));

    // ===== P3 双跑归因分解表（逐项开关对本拓扑 free 值贡献）=====
    emit(format!("  [attribution {w}x{h} free-zone]"));
    emit(format!(
        "    full new pipeline (criterion loss + dense ladder + independent seeds): free={}‰",
        rp.free_flow_max_err_permille.min(r.free_flow_max_err_permille)
    ));
    let ctrl = fit_rational_rel_sse_control(&fit_samples);
    let r_ctrl = evaluate_rational_fit(&ctrl, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "    objective=rel-SSE control (old target, same samples): free={}‰ (Δ={})",
        r_ctrl.free_flow_max_err_permille,
        (r_ctrl.free_flow_max_err_permille as i64)
            - (r.free_flow_max_err_permille as i64)
    ));
    // 梯度密度因子：旧阶梯（2→256 倍增 8 档）+ 新目标。
    let old_ladder: [u64; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
    let (old_samples, _) = sample_ladder(w, h, &old_ladder, 0);
    let (fit_old_ladder, _, bind_ol) =
        fit_rational_saturation_detailed(&old_samples, &ZoneBands::SET_D);
    let r_ol = evaluate_rational_fit(&fit_old_ladder, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "    ladder=old 8-step (new objective): free={}‰ binding={} (Δ={})",
        r_ol.free_flow_max_err_permille,
        bind_ol,
        (r_ol.free_flow_max_err_permille as i64) - (r.free_flow_max_err_permille as i64)
    ));
    // 取整因子：|q_round − q_floor| 上界逐点最大（≤1‰ 结构性）。
    let mut round_max = 0u64;
    for s in &holdout {
        let num = pfit.q_max_permille as u128 * s.k_permille as u128;
        let den = ((s.k_permille as u128).pow(2) + (pfit.k0_permille as u128).pow(2)).isqrt().max(1);
        let q_floor = (num / den) as u64;
        let q_round = q_pnorm_permille(&pfit, s.k_permille);
        round_max = round_max.max(q_floor.abs_diff(q_round));
    }
    emit(format!(
        "    rounding (|round−floor| per point, pnorm-p2): max {round_max}‰ (structural ≤1‰)"
    ));
    // back-half 种子因子：独立段 vs 旧误用 holdout 段（+8..16）。
    let old_bug_hold: Vec<MfdSample> = HOLDOUT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SEED_BASE + 8 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let (fit_ob, _, bind_ob) = fit_rational_saturation_detailed(&back_fit, &ZoneBands::SET_D);
    let r_ob = evaluate_rational_fit(&fit_ob, &old_bug_hold, &ZoneBands::SET_D);
    emit(format!(
        "    back-half seeds=reused holdout segment (old bug emulation): free={}‰ binding={} vs independent {free_ind}‰",
        r_ob.free_flow_max_err_permille,
        bind_ob,
        free_ind = r_b.free_flow_max_err_permille
    ));
    outcome_pass
}

fn main() {
    let mut out: Vec<String> = Vec::new();
    out.push(
        "②′ confirmation round (debt #31; pre-registered 7160955; one-shot termination: any FAIL -> disposition ① automatically)"
            .to_string(),
    );
    out.push(format!(
        "evidence commit: {}",
        option_env!("SETD_COMMIT").unwrap_or("(unset — build with SETD_COMMIT env)")
    ));
    out.push(format!(
        "criteria: free<={}‰ transition<={}‰ saturated<={}‰ (single source = sim-traffic consts); families frozen = {{rational, pnorm-p2}}",
        FREE_FLOW_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE
    ));
    // 结构化终止判定（审计3 C-2：由返回的布尔值驱动，不嗅探文本）。
    let failed = !(report_topology(40, 40, &mut out)
        & report_topology(32, 32, &mut out)
        & report_topology(64, 64, &mut out));
    out.push(format!(
        "②′ AUTO-DISPOSITION: {}",
        if failed {
            "FAIL -> disposition ① (追认转 C, MFD 永久纯统计) — applied automatically per pre-registered termination clause"
        } else {
            "PASS -> #31 核销、3a 记通过（解冻另走 ADR-NEW-26 评审）"
        }
    ));
    for l in &out {
        println!("{l}");
    }
}
