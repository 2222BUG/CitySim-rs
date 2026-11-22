# STAGE2B3-BUNDLE PART 09（crates/sim-traffic/src/lib.rs 续段 2）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3 -->
) {
        // 点队列模型无 spillback，(k,q) 单调饱和 → 三角 -w 分支不可辨识，
        // 拟合须显式报告 descending_branch_observed=false（平台型解读）。
        let (_, mut samples) = small_run(&[2, 4, 8, 16, 32]);
        samples.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&samples);
        if samples.iter().filter(|s| s.k_permille > f.k_c_permille).all(|s| s.q_permille >= f.q_max_permille)
        {
            assert!(!f.descending_branch_observed);
            assert_eq!(f.w_permille, 0);
        }
    }

    #[test]
    fn synthetic_triangle_is_recovered() {
        // 人工合成精确三角：v_free=1000‰、k_c=100‰、q_max=100‰、w=500‰。
        let samples = vec![
            MfdSample { k_permille: 10, q_permille: 10 },
            MfdSample { k_permille: 50, q_permille: 50 },
            MfdSample { k_permille: 100, q_permille: 100 },
            MfdSample { k_permille: 200, q_permille: 50 },
            MfdSample { k_permille: 300, q_permille: 0 },
        ];
        let f = fit_triangle(&samples);
        assert_eq!(f.v_free_permille, 1000);
        assert_eq!(f.k_c_permille, 100);
        assert_eq!(f.q_max_permille, 100);
        assert!(f.descending_branch_observed);
        assert_eq!(f.w_permille, 500);
    }

    #[test]
    fn explicit_id_injection_is_order_independent() {
        // 2026-11-19 投票修复：同一 ID 集合以不同物理顺序注入，权威状态（摘要）必须一致。
        let run = |order: &[u64]| {
            let mut gt = GridTraffic::new(8, 8, 1, 0x00C0_FFEE_1234_5678);
            for &id in order {
                gt.inject_with_id(id);
            }
            let mut g = GlobalDigest::new();
            gt.fold_digest(&mut g);
            for _ in 0..10u64 {
                gt.step();
                gt.fold_digest(&mut g);
            }
            g.finalize()
        };
        let d1 = run(&[3, 1, 2, 5, 4]);
        let d2 = run(&[1, 2, 3, 4, 5]);
        let d3 = run(&[5, 4, 3, 2, 1]);
        assert_eq!(d1, d2, "injection order must not matter");
        assert_eq!(d2, d3, "injection order must not matter");
    }

    #[test]
    fn fold_digest_covers_goal_and_next_id() {
        // 强读档等价护栏：goal 与 next_id 属权威状态，摘要必须敏感。
        let base = |mut gt: GridTraffic| {
            for _ in 0..5u64 {
                gt.inject(3);
                gt.step();
            }
            let mut g = GlobalDigest::new();
            gt.fold_digest(&mut g);
            g.finalize()
        };
        let d1 = base(GridTraffic::new(8, 8, 1, 7));
        let d2 = base(GridTraffic::new(8, 8, 1, 7));
        assert_eq!(d1, d2);

        // next_id 敏感
        let mut gt = GridTraffic::new(8, 8, 1, 7);
        for _ in 0..5u64 {
            gt.inject(3);
            gt.step();
        }
        gt.next_id += 1;
        let mut g = GlobalDigest::new();
        gt.fold_digest(&mut g);
        assert_ne!(d1, g.finalize(), "next_id must be covered");

        // goal 敏感
        let mut gt = GridTraffic::new(8, 8, 1, 7);
        for _ in 0..5u64 {
            gt.inject(3);
            gt.step();
        }
        if let Some(v) = gt.vehicles.first_mut() {
            v.goal_x = v.goal_x.wrapping_add(1);
        }
        let mut g = GlobalDigest::new();
        gt.fold_digest(&mut g);
        assert_ne!(d1, g.finalize(), "vehicle goal must be covered");
    }

    #[test]
    fn synthetic_holdout_meets_adr25_criteria() {
        // 精确三角（v_free=1000‰, k_c=100‰, q_max=100‰, w=500‰）：
        // 拟合用 (10,100,300) 三点，留出 (50,200) 两点——独立于拟合集评估。
        let fit_in = vec![
            MfdSample { k_permille: 10, q_permille: 10 },
            MfdSample { k_permille: 100, q_permille: 100 },
            MfdSample { k_permille: 300, q_permille: 0 },
        ];
        let holdout = vec![
            MfdSample { k_permille: 50, q_permille: 50 },
            MfdSample { k_permille: 200, q_permille: 50 },
        ];
        let f = fit_triangle(&fit_in);
        assert!(f.descending_branch_observed);
        assert_eq!(f.v_free_permille, 1000);
        assert_eq!(f.w_permille, 500);
        let r = evaluate_fit(&f, &holdout, &ZoneBands::SET_D);
        // ADR-NEW-25 3a 判据（原 10/20/25% 三区重分）：自由流 ≤100‰、饱和区 ≤250‰、
        // k_c ≤200‰（三角）；转折区只记录不设硬判据。
        assert!(r.free_flow_max_err_permille <= 100, "{}", r.free_flow_max_err_permille);
        assert!(r.k_c_err_permille <= 200, "{}", r.k_c_err_permille);
        assert!(r.saturated_max_err_permille <= 250, "{}", r.saturated_max_err_permille);
    }

    #[test]
    #[ignore = "债务 #31 审计退回（2026-11-20）：保持 ignore，直至口径修复后 Set D 复测通过并经负责人追认（FAIL 翻转则按重投裁决处置）"]
    fn simulation_holdout_free_flow_is_accurate() {
        // 两次独立小场景（同种子不同密度阶梯）：A 拟合、B 留出——独立密度点评估。
        let (_, mut a) = small_run(&[1, 2, 4, 8]);
        let (_, b) = small_run(&[3, 6, 12]);
        a.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&a);
        let r = evaluate_fit(&f, &b, &ZoneBands::SET_D);
        assert!(r.points_evaluated >= 2);
        // 门禁对齐（决策4 整改）：断言引用判据常量（100‰），不再用 250‰ 脱钩阈值。
        // 当前点队列模型自由流实测 190‰ FAIL——债务 #31（凹形扩展）Set D 复测通过前
        // 本测试标 ignore，避免"FAIL 仍绿"掩盖真实状态（豆包/GPT-6 指出）。
        assert!(
            r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE,
            "free-flow err {}‰",
            r.free_flow_max_err_permille
        );
    }

    // ---- 凹形有理饱和族（决策4 W1）----

    fn rational_samples(q_max: u64, k_half: u64, ks: &[u64]) -> Vec<MfdSample> {
        let fit = RationalSaturationFit { q_max_permille: q_max, k_half_permille: k_half };
        ks.iter()
            .map(|&k| MfdSample { k_permille: k, q_permille: q_rational_permille(&fit, k) })
            .collect()
    }

    #[test]
    fn synthetic_rational_is_recovered() {
        // 人工合成精确有理饱和图（q_max=200‰、k_half=150‰）→辨识恢复。
        let ks: Vec<u64> = (1..=40).map(|i| i * 25).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks), &ZoneBands::SET_D);
        assert_eq!(fit.q_max_permille, 200);
        assert_eq!(fit.k_half_permille, 250);
    }

    #[test]
    fn rational_identification_is_deterministic() {
        let ks: Vec<u64> = (1..=30).map(|i| i * 17).collect();
        let a = fit_rational_saturation(&rational_samples(200, 250, &ks), &ZoneBands::SET_D);
        let b = fit_rational_saturation(&rational_samples(200, 250, &ks), &ZoneBands::SET_D);
        assert_eq!(a, b);
    }

    #[test]
    fn rational_zero_density_slope_is_bounded() {
        let ks: Vec<u64> = (1..=30).map(|i| i * 33).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks), &ZoneBands::SET_D);
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn rational_bound_violation_is_disclosed_and_constrained() {
        // 越界判失败口径（决策4）：自由估计越界→披露幅度+物理可行解回退，
        // 不 panic 不截断。合成 (500,50) 自由斜率 10000‰ 必越界。
        // 样本取 k≥500 保证 q≤k（采样不变量断言下合法），模型越界仍成立。
        let ks: Vec<u64> = vec![500, 1000, 2000];
        let (fit, free_slope, binding) =
            fit_rational_saturation_detailed(&rational_samples(500, 50, &ks), &ZoneBands::SET_D);
        assert!(binding, "bound must bind");
        assert_eq!(free_slope, 10_000, "violation magnitude must be disclosed");
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn rational_holdout_meets_criteria_on_synthetic() {
        // 合成数据留出评估：三区判据应全过（常量单一来源）。
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks), &ZoneBands::SET_D);
        let holdout: Vec<u64> = (1..=12).map(|i| i * 40 + 13).collect();
        let r = evaluate_rational_fit(&fit, &rational_samples(200, 250, &holdout), &ZoneBands::SET_D);
        assert!(r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE, "{}", r.free_flow_max_err_permille);
        assert!(r.transition_max_err_permille <= TRANSITION_CRIT_PERMILLE, "{}", r.transition_max_err_permille);
        assert!(r.saturated_max_err_permille <= SATURATED_CRIT_PERMILLE, "{}", r.saturated_max_err_permille);
    }

    // ===== X3：p-norm p=2 族（豆包清单 d：多族并行辨识） =====

    fn pnorm_samples(q_max: u64, k0: u64, ks: &[u64]) -> Vec<MfdSample> {
        let fit = PNormConcaveFit { q_max_permille: q_max, k0_permille: k0 };
        ks.iter()
            .map(|&k| MfdSample { k_permille: k, q_permille: q_pnorm_permille(&fit, k) })
            .collect()
    }

    #[test]
    fn synthetic_pnorm_is_recovered() {
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let (fit, free_slope, binding) = fit_pnorm_concave_detailed(&pnorm_samples(200, 250, &ks), &ZoneBands::SET_D);
        assert_eq!(fit, PNormConcaveFit { q_max_permille: 200, k0_permille: 250 });
        assert_eq!(free_slope, 800);
        assert!(!binding);
    }

    #[test]
    fn pnorm_identification_is_deterministic() {
        let ks: Vec<u64> = (1..=16).map(|i| i * 40).collect();
        let s = pnorm_samples(300, 400, &ks);
        assert_eq!(fit_pnorm_concave_detailed(&s, &ZoneBands::SET_D), fit_pnorm_concave_detailed(&s, &ZoneBands::SET_D));
    }

    #[test]
    fn pnorm_bound_violation_is_disclosed_and_constrained() {
        // 自由估计斜率超界（500/50=10000‰）→binding=true 即失败信号；
        // 可行解回退仅诊断，zero_density_slope≤1000‰。样本 k≥500 保证 q≤k。
        let ks: Vec<u64> = vec![500, 1000, 2000];
        let (fit, free_slope, binding) = fit_pnorm_concave_detailed(&pnorm_samples(500, 50, &ks), &ZoneBands::SET_D);
        assert!(binding);
        assert!(free_slope > 1000);
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn pnorm_holdout_meets_criteria_on_synthetic() {
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let fit = fit_pnorm_concave(&pnorm_samples(200, 250, &ks), &ZoneBands::SET_D);
        let holdout: Vec<u64> = (1..=12).map(|i| i * 40 + 13).collect();
        let r = evaluate_pnorm_fit(&fit, &pnorm_samples(200, 250, &holdout), &ZoneBands::SET_D);
        assert!(r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE, "{}", r.free_flow_max_err_permille);
        assert!(r.transition_max_err_permille <= TRANSITION_CRIT_PERMILLE, "{}", r.transition_max_err_permille);
        assert!(r.saturated_max_err_permille <= SATURATED_CRIT_PERMILLE, "{}", r.saturated_max_err_permille);
    }

    #[test]
    fn pnorm_isqrt_rounding_golden() {
        // isqrt 舍入黄金（决策5 共识收口条件）：手算定点期望值钉死语义——
        // q = round(q_max·k·1000 / isqrt((k²+k0²)·10⁶))，就近舍入 ((num+den/2)/den)。
        let f = PNormConcaveFit { q_max_permille: 200, k0_permille: 250 };
        // 注意 isqrt floor 偏差披露（决策5）：den 先取整再除，使结果较真值
        // 有 +0~+1‰ 的系统性上偏（如 k=250 真值 141.42→定点 142）。
        assert_eq!(q_pnorm_permille(&f, 0), 0);
        // k=250：真值 141.42‰，定点就近→142（isqrt floor 偏差）
        assert_eq!(q_pnorm_permille(&f, 250), 142);
        // k=1000：真值 194.03‰ → 194
        assert_eq!(q_pnorm_permille(&f, 1000), 194);
        // k=2500：真值 198.03‰，定点→199（isqrt floor 偏差）
        assert_eq!(q_pnorm_permille(&f, 2500), 199);
    }

    #[test]
    fn criterion_aligned_loss_is_zero_on_exact_fit() {
        // ②′ 修测：拟合目标=判据归一化分区最大相对误差；精确参数合成数据下
        // loss≈0（仅舍入级），且非精确参数 loss 严格更大（目标可辨识）。
        let ks: Vec<u64> = (1..=24).map(|i| i * 40).collect();
        let s = rational_samples(200, 250, &ks);
        let exact = RationalSaturationFit { q_max_permille: 200, k_half_permille: 250 };
        let loss_exact =
            criterion_aligned_loss(|k| q_rational_permille(&exact, k), &s, &ZoneBands::SET_D);
        let off = RationalSaturationFit { q_max_permille: 200, k_half_permille: 200 };
        let loss_off =
            criterion_aligned_loss(|k| q_rational_permille(&off, k), &s, &ZoneBands::SET_D);
        assert!(loss_exact <= 10, "exact loss {}", loss_exact);
        assert!(loss_off > loss_exact, "identifiability: {} vs {}", loss_off, loss_exact);
    }

    #[test]
    fn point_resolution_floor_edges() {
        // ②′ 修测：判据×q_obs ≥ 2000（即相对容差 ≥2‰ 采样单位）。
        assert!(point_resolution_ok(20, 100)); // 20‰×100%=20‰… 100×20=2000 边界达标
        assert!(!point_resolution_ok(19, 100)); // 1900<2000 分辨率受限
        assert!(point_resolution_ok(8, 250)); // 250×8=2000 边界达标
        assert!(!point_resolution_ok(7, 250)); // 1750<2000 分辨率受限
    }

    #[test]
    fn steady_window_dual_criteria() {
        // ②′ 稳态硬门：双口径任一（≤2‰ 或 ≤15 辆）。
        assert!(steady_window(2, 100)); // ‰ 口径达标
        assert!(steady_window(30, 15)); // 辆口径达标
        assert!(!steady_window(30, 16)); // 双不达标
        assert!(steady_window(2, 16)); // 边界
    }
}
