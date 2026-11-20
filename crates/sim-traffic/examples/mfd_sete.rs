//! Set D 复测（债务 #31 核销条件；ADR-NEW-25 3a 决策4 判据修订 (c)）。
//! Set E 确认轮（决策5 收口条件）：新种子段 0x00C0_FFEE_3333_0000、
//! fit/holdout 种子不共用（fit +0..7 / holdout +8..15）、第三拓扑 32x32、
//! 族集合与选族规则已于 c0f94ba 冻结——本文件先落盘后取数（预注册）。
//!
//! 预注册 fixture（**运行前声明，先于 Set D 数据**）：
//! - Set D 种子段=0x00C0_FFEE_2222_0000 + i（i=0..8），与 V1 的 A/B/C 段
//!   （0x00C0_FFEE_1234_5678+i）不相交；
//! - 拓扑：主 40×40 + 第二拓扑 64×64（判据修订要求）；
//! - 密度阶梯：拟合集 2→256 倍增 8 档；留出集 3→384 倍增 8 档（交错不相交）；
//! - 每档 260 tick，末 60 tick 窗口均值；稳态判据=窗口前后半 q 差 ≤2‰；
//! - 每档 8 种子平均；分区边界=ZoneBands::SET_D 预注册绝对带（k<100 / 100..2000 /
//!   ≥2000‰，禁由拟合参数导出——审计修复 2026-11-20，决策4 3a(a)）；
//! - 判据（常量单一来源=sim-traffic）：自由流 ≤100‰、转折 ≤250‰、饱和 ≤250‰。
//!
//! 终止条件（Claude）：任一拓扑任一区 FAIL → 转 C，不再延后。
//!
//! 运行：cargo run --release --example mfd_setd -p sim-traffic

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use sim_traffic::{
    evaluate_pnorm_fit, evaluate_rational_fit, fit_pnorm_concave_detailed,
    fit_rational_saturation_detailed, q_pnorm_permille, q_rational_permille, FitErrorReport,
    GridTraffic,
    MfdSample, ZoneBands, FREE_FLOW_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE,
};

const SETD_SEED_BASE: u64 = 0x00C0_FFEE_3333_0000;
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
            // q≤k fail-fast（审计修复 2026-11-20）：同群体口径（last_step_population
            // 含到达车末步，与 flow 同群体）下每车每 tick ≤1 条边。
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
    // 稳态判据双口径：相对差 ≤2‰，或绝对差 ≤ 半窗口一半（≈0.5 车/tick）——
    // 小流量点 1 辆车波动即超 2‰，纯相对口径是测量分辨率问题非发散（W2 实测教训）。
    let steady = q_front.abs_diff(q_back) * 1000 <= (q_front + q_back) * 2
        || q_front.abs_diff(q_back) <= (WINDOW as u64) / 4;
    (k_sum / n, q_sum / n, steady)
}

/// 稳态敏感性采样（审计修复 2026-11-20，DeepSeek A5/MiniMax A5）：仅窗口后半
/// （t >= TICKS − WINDOW/2）采样，返回 (k̄, q̄)。
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

/// 多种子平均采样。稳态仅作诊断（不阻断）：返回 (样本, 未稳态档数)。
fn sample_ladder(w: u32, h: u32, ladder: &[u64], seed_off: u64) -> (Vec<MfdSample>, usize) {
    let mut unsteady = 0usize;
    let samples = ladder
        .iter()
        .map(|&inj| {
            let (mut k, mut q, mut steady) = (0u64, 0u64, true);
            for i in 0..N_SEEDS {
                let (k1, q1, s1) = run_density_seed(w, h, inj, SETD_SEED_BASE + seed_off + i);
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

/// 相对误差平方和（‰口径）——族比较统一目标（与判据同构）。
/// 模型函数表项（clippy: type_complexity 收敛；生命周期须带出 'static 默认）。
type QModelFn<'a> = Box<dyn Fn(u64) -> u64 + 'a>;
fn rel_sse(
q: impl Fn(u64) -> u64, samples: &[MfdSample]) -> u128 {
    let mut sse = 0u128;
    for s in samples {
        let e = q(s.k_permille) as i128 - s.q_permille as i128;
        let er = (e * 1000) / (s.q_permille.max(1) as i128);
        sse += (er * er) as u128;
    }
    sse
}

/// 单一判定核（决策5 整改：合并原 family_verdict/topology_fail 双实现——
/// 双实现漂移教训）。越界（binding）即 FAIL（决策4 条件⑧）；三区判据常量
/// 单一来源，**含转折区硬判据**（决策4(b)——决策5 五家行号命中的漏接线缺陷）；
/// 空区即 FAIL（决策4(d) 样本数下限：防空区真空 PASS；<2 点区在输出行标 n 值披露）。
fn family_verdict(binding: bool, r: &FitErrorReport) -> &'static str {
    if binding
        || r.free_flow_max_err_permille > FREE_FLOW_CRIT_PERMILLE
        || r.transition_max_err_permille > TRANSITION_CRIT_PERMILLE
        || r.saturated_max_err_permille > SATURATED_CRIT_PERMILLE
        || r.free_n == 0
        || r.trans_n == 0
        || r.sat_n == 0
    {
        "FAIL"
    } else {
        "PASS"
    }
}

fn report_topology(w: u32, h: u32, emit: &mut dyn FnMut(String)) {
    emit(format!("=== topology {w}x{h} (Set E seeds {SETD_SEED_BASE:#x}+0..{N_SEEDS}) ==="));
    let (mut fit_samples, unsteady_fit) = sample_ladder(w, h, &FIT_LADDER, 0);
    fit_samples.sort_by_key(|s| s.k_permille);
    let (holdout, unsteady_hold) = sample_ladder(w, h, &HOLDOUT_LADDER, 8);
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
    // ===== 决策5 Y1：选族前置（Q3 实装）——先辨识两族、只用拟合集选族并锁定，
    // 之后才生成留出表与评估（计算与打印顺序一致，防"先见留出"误读，Kimi）。
    let (pfit, p_slope, p_binding) = fit_pnorm_concave_detailed(&fit_samples);
    // 拟合集逐点归档（决策5 MiniMax/豆包 R5：预选所依据的 rel-SSE 须可复核）。
    let qf_tab: Vec<(&str, QModelFn<'_>)> = vec![
        ("rational", Box::new(|k: u64| q_rational_permille(&fit, k))),
        ("pnorm-p2", Box::new(|k: u64| q_pnorm_permille(&pfit, k))),
    ];
    for (tag, qf) in &qf_tab {
        for s in &fit_samples {
            let qm = qf(s.k_permille);
            let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000 / (s.q_permille.max(1) as u128);
            emit(format!(
                "  [fit-set {tag}] k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ err {err}‰",
                s.k_permille, s.q_permille, qm
            ));
        }
    }
    let sse_r = rel_sse(|k| q_rational_permille(&fit, k), &fit_samples);
    let sse_p = rel_sse(|k| q_pnorm_permille(&pfit, k), &fit_samples);
    // Q3 预选（决策5 全票通过）：物理可行域（binding=false）内拟合集 rel-SSE 最小；
    // 越界族不具参赛资格（条件⑧族级读法：被判失败者不得当选）；全部越界→无族
    // 可选（族级 FAIL 兜底，Grok/Gemini）；一次锁定不得回退（豆包）。
    let selected: Option<&str> = match (binding, p_binding) {
        (true, true) => None,
        (false, true) => Some("rational"),
        (true, false) => Some("pnorm-p2"),
        (false, false) => Some(if sse_p < sse_r { "pnorm-p2" } else { "rational" }),
    };
    emit(format!(
        "  [family pre-selection] fit-set rel-SSE rational={sse_r} pnorm={sse_p} binding(r={binding},p={p_binding}) -> selected={} (Q3 feasible-domain rule; locked before holdout)",
        selected.unwrap_or("NONE (all binding)")
    ));
    // 留出集逐点表（证据展示；选族已锁定，不再影响 selected）
    for s in &holdout {
        let qm = q_rational_permille(&fit, s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000 / (s.q_permille.max(1) as u128);
        emit(format!(
            "  k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ err {}‰",
            s.k_permille, s.q_permille, qm, err
        ));
    }
    let r = evaluate_rational_fit(&fit, &holdout, &ZoneBands::SET_D);
    let verdict = |v: u64, c: u64| if v <= c { "PASS" } else { "FAIL" };
    // 越界判失败（决策4 条件⑧，审计修复 2026-11-20）：自由估计越界→本拓扑直接
    // FAIL；binding 回退解仅诊断披露，不参与 PASS/FAIL 判定。
    if binding {
        emit(format!(
            "  bound violation: free estimate slope {}‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)",
            free_slope
        ));
    }
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
    // 单一判定核输出（含转折硬判据+空区 FAIL+各区 n 披露，决策5 整改）
    emit(format!(
        "  topology verdict [rational] = {} (binding={} free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}))",
        family_verdict(binding, &r), binding,
        r.free_flow_max_err_permille, r.free_n,
        r.transition_max_err_permille, r.trans_n,
        r.saturated_max_err_permille, r.sat_n
    ));
    // ===== pnorm 留出评估（选族已在前置块锁定，此处只评估+终局）=====
    emit(format!(
        "  [family pnorm-p2] fit: q_max={}‰ k0={}‰ slope free={}‰ binding={}",
        pfit.q_max_permille, pfit.k0_permille, p_slope, p_binding
    ));
    let rp = evaluate_pnorm_fit(&pfit, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "  [family pnorm-p2] free={}‰ (n={}) trans={}‰ (n={}) sat={}‰ (n={}) -> {}",
        rp.free_flow_max_err_permille, rp.free_n, rp.transition_max_err_permille, rp.trans_n,
        rp.saturated_max_err_permille, rp.sat_n,
        family_verdict(p_binding, &rp)
    ));
    emit(format!(
        "  Set E outcome (selected family {}) = {}",
        selected.unwrap_or("NONE"),
        match selected {
            Some("pnorm-p2") => family_verdict(p_binding, &rp),
            Some(_) => family_verdict(binding, &r),
            None => "FAIL (no feasible family: all binding)",
        }
    ));
    // 稳态敏感性（审计修复 2026-11-20）：窗口后半单独采样复算三区指标，
    // 若与全窗结论一致则"稳态仅披露"升级为"已验证稳健"。
    let back_fit: Vec<MfdSample> = FIT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + 8 + i);
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
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + 8 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let (fit_b, _, binding_b) = fit_rational_saturation_detailed(&back_fit);
    let r_b = evaluate_rational_fit(&fit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [sensitivity back-half rational] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        r_b.free_flow_max_err_permille, r_b.free_n,
        r_b.transition_max_err_permille, r_b.trans_n,
        r_b.saturated_max_err_permille, r_b.sat_n,
        binding_b,
        family_verdict(binding_b, &r_b)
    ));
    // 决策5 共识收口条件：pnorm back-half 敏感性（此前只测 rational）。
    let (pfit_b, _, p_binding_b) = fit_pnorm_concave_detailed(&back_fit);
    let rp_b = evaluate_pnorm_fit(&pfit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [sensitivity back-half pnorm-p2] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        rp_b.free_flow_max_err_permille, rp_b.free_n,
        rp_b.transition_max_err_permille, rp_b.trans_n,
        rp_b.saturated_max_err_permille, rp_b.sat_n,
        p_binding_b,
        family_verdict(p_binding_b, &rp_b)
    ));
}

fn main() {
    let mut out: Vec<String> = Vec::new();
    out.push("Set D re-verification (debt #31; pre-registered fixture, disjoint seeds)".to_string());
    // 证据-源码同源标注（审计修复 2026-11-20）：证据文件自带生成它的 commit hash
    // （编译期捕获：运行前设 SETD_COMMIT=$(git rev-parse HEAD) 并重编译）。
    out.push(format!(
        "evidence commit: {}",
        option_env!("SETD_COMMIT").unwrap_or("(unset — build with SETD_COMMIT env)")
    ));
    out.push(format!(
        "criteria: free<={}‰ transition<={}‰ saturated<={}‰ (single source = sim-traffic consts)",
        FREE_FLOW_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE
    ));
    report_topology(40, 40, &mut |l| out.push(l));
    report_topology(32, 32, &mut |l| out.push(l));
    report_topology(64, 64, &mut |l| out.push(l));
    for l in &out {
        println!("{l}");
    }
}
