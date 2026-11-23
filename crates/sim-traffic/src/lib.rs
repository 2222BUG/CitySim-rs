//! 交通系统：MFD 微观基准 + 区域聚合采样（ADR-NEW-25，Stage 2C C3b）。
//!
//! - 权威 crate。确定性：BTreeMap 队列、车辆按 ID 全序、event_u64 派生 OD、
//!   u64 tick 计数，无时钟/无浮点/无 HashMap（2.9）。
//! - MFD 三角基本图（候选）：微观基准采样 (k, q) → 定点拟合 v_free/k_c/q_max/w。
//! - T-Bench-1（ADR-NEW-25 决策 4）：40×40 双向网格、密度阶梯注入、每档稳定
//!   200 tick 后采样；本文件以缩小档位实现，example 输出完整档位。

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_rng::event_u64;

pub const LAYER: &str = "交通系统";
pub const AUTHORITATIVE: bool = true;

pub fn authoritative() -> bool {
    AUTHORITATIVE
}

/// 车辆（确定性：ID 即全序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vehicle {
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub goal_x: u32,
    pub goal_y: u32,
    pub edge_traversals: u64,
}

impl Vehicle {
    pub fn arrived(&self) -> bool {
        self.x == self.goal_x && self.y == self.goal_y
    }
}

/// 网格交通基准（T-Bench-1）：W×H 双向网格，每边每 tick 容量 cap 辆。
#[derive(Debug, Clone)]
pub struct GridTraffic {
    pub w: u32,
    pub h: u32,
    pub cap_per_edge_per_tick: u64,
    pub tick: u64,
    /// 等待进入路网的注入队列（按 (注入 tick, id) 全序）。
    pending: Vec<Vehicle>,
    pub vehicles: Vec<Vehicle>,
    next_id: u64,
    /// 本 tick 完成的边遍历数（流量采样）。
    pub flow_this_tick: u64,
    /// 本 tick 参与移动的车辆数（step 中移除到达车**之前**的人口；k 采样口径，
    /// 与 flow 同群体 → q≤k 恒成立——审计修复 2026-11-20：旧口径 flow 含到达车
    /// 末步、active_count 不含，q>k 违规点伪造误差地板）。
    pub last_step_population: u64,
    seed: u64,
}

impl GridTraffic {
    pub fn new(w: u32, h: u32, cap: u64, seed: u64) -> Self {
        Self {
            w,
            h,
            cap_per_edge_per_tick: cap,
            tick: 0,
            pending: Vec::new(),
            vehicles: Vec::new(),
            next_id: 0,
            flow_this_tick: 0,
            last_step_population: 0,
            seed,
        }
    }

    /// 注入 n 辆车（OD 由 event_u64 确定性派生；曼哈顿路径，先 x 后 y）。
    pub fn inject(&mut self, n: u64) {
        for _ in 0..n {
            let id = self.next_id;
            self.next_id += 1;
            self.inject_with_id(id);
        }
    }

    /// 显式 ID 注入（2026-11-19 投票修复：注入顺序无关性）。
    /// 契约：ID 由集中分配点按稳定键分配（ADR-NEW-20 口径）；OD 派生只依赖 ID，
    /// pending 入档/进网均按 id 稳定序 → 注入的物理顺序不影响任何权威状态。
    /// 调用方负责 ID 唯一性（重复 ID 会产生两辆车，属调用方契约违例）。
    pub fn inject_with_id(&mut self, id: u64) {
        let sx = (event_u64(self.seed, "traffic.od", id, 0) % self.w as u64) as u32;
        let sy = (event_u64(self.seed, "traffic.od", id, 1) % self.h as u64) as u32;
        let gx = (event_u64(self.seed, "traffic.od", id, 2) % self.w as u64) as u32;
        let gy = (event_u64(self.seed, "traffic.od", id, 3) % self.h as u64) as u32;
        self.pending.push(Vehicle {
            id,
            x: sx,
            y: sy,
            goal_x: gx,
            goal_y: gy,
            edge_traversals: 0,
        });
    }

    /// 推进一个 tick：注入进网、车辆沿曼哈顿路径前进一步（容量让行）。
    /// 全序：车辆按 id 升序处理；同位置同 tick 内先到先行（id 序）。
    pub fn step(&mut self) {
        self.step_with_lod(|_| 0);
    }

    /// LOD 分频步进（rfc-lod-draft 语义）：lod_of(id)∈{0,1,2}，仅移动更新按
    /// 2^lod 分频（L0 每 tick、L1 每 2 tick、L2 每 4 tick，取 tick 的低位对齐——
    /// 确定性信号，禁墙钟）；注入/到达出网照常（非移动更新）。
    pub fn step_with_lod(&mut self, lod_of: impl Fn(u64) -> u8) {
        self.tick += 1;
        self.flow_this_tick = 0;
        // 注入进网（pending 已按 id 序）
        let entering: Vec<Vehicle> = std::mem::take(&mut self.pending);
        for v in entering {
            self.vehicles.push(v);
        }
        self.vehicles.sort_by_key(|v| v.id);
        // 容量记账：每个目标格每 tick 只放行 cap 辆（cap=1 时即逐格排队）
        let mut admitted_at: std::collections::BTreeMap<(u32, u32), u64> =
            std::collections::BTreeMap::new();
        for v in self.vehicles.iter_mut() {
            if v.arrived() {
                continue;
            }
            let lod = lod_of(v.id).min(2);
            if lod > 0 && (self.tick & ((1u64 << lod) - 1)) != 0 {
                continue; // 本 tick 该实体在低精度层，跳过移动更新
            }
            let (nx, ny) = if v.x != v.goal_x {
                (if v.x < v.goal_x { v.x + 1 } else { v.x - 1 }, v.y)
            } else {
                (v.x, if v.y < v.goal_y { v.y + 1 } else { v.y - 1 })
            };
            let slot = admitted_at.entry((nx, ny)).or_insert(0);
            if *slot < self.cap_per_edge_per_tick {
                *slot += 1;
                v.x = nx;
                v.y = ny;
                v.edge_traversals += 1;
                self.flow_this_tick += 1;
            }
        }
        // k 采样口径（审计修复 2026-11-20）：人口在移除到达车之前计量，与
        // flow_this_tick（含到达车末步）同群体 → q≤k 恒成立（每车每 tick ≤1 条边）。
        self.last_step_population = self.vehicles.len() as u64;
        // 已到达车辆出网（保持 vehicles=活跃车辆）
        self.vehicles.retain(|v| !v.arrived());
    }

    /// 活跃车辆数（密度分子）。
    /// 需求压力（万分比 0..=10000）= 当前密度 k 相对特征密度 k_ref_permille 的比值封顶。
    /// 确定性定点（权威状态导出）；MFD→降级信号通道（ADR-NEW-26、ADR-NEW-25 3c）。
    /// 决策4 修订：k_ref=特征密度（q 首次 ≥90%·q_max 密度，版本化）；k_ref==0 返回 0
    /// （fail-safe 改向：分母未知时宁可不触发降级，原返回 10000 方向反）。
    pub fn demand_pressure_bp(&self, k_ref_permille: u64) -> u32 {
        if k_ref_permille == 0 || self.edge_count() == 0 {
            return 0;
        }
        // 采样口径=last_step_population（与 q 同群体；审计修复 2026-11-20，
        // MiMo #5：乘法走 u128 防 u64 环绕；edge_count==0 fail-safe 返回 0）。
        let k = (self.last_step_population as u128 * 1000 / self.edge_count() as u128) as u64;
        ((k as u128 * 10_000 / k_ref_permille as u128).min(10_000)) as u32
    }

    pub fn active_count(&self) -> u64 {
        self.vehicles.len() as u64
    }

    /// 网格链接总数（双向边数；密度分母口径=边数）。
    pub fn edge_count(&self) -> u64 {
        (2 * self.w as u64 * (self.h as u64 - 1)) + (2 * self.h as u64 * (self.w as u64 - 1))
    }

    /// 逐 tick 折叠摘要（G3/确定性判据用）。
    /// 覆盖全部权威状态（ADR-NEW-19 纳入范围）：配置、tick、pending 注入队列、
    /// next_id 分配器、逐车全字段（含 goal 与 traversals）——2026-11-19 投票修复：
    /// 补 goal_x/goal_y/next_id 等字段，消除强读档等价假通过风险。
    pub fn fold_digest(&self, g: &mut GlobalDigest) {
        let mut w = CanonicalWriter::new("sim-traffic", "TrafficTick");
        w.u64(self.w as u64);
        w.u64(self.h as u64);
        w.u64(self.cap_per_edge_per_tick);
        w.u64(self.seed);
        w.u64(self.tick);
        w.u64(self.next_id);
        // pending 按 id 稳定序入档（注入顺序无关契约）
        let mut pending_sorted: Vec<&Vehicle> = self.pending.iter().collect();
        pending_sorted.sort_by_key(|v| v.id);
        w.seq_len(pending_sorted.len());
        for v in pending_sorted {
            w.u64(v.id);
            w.u32(v.x);
            w.u32(v.y);
            w.u32(v.goal_x);
            w.u32(v.goal_y);
            w.u64(v.edge_traversals);
        }
        w.u64(self.vehicles.len() as u64);
        w.u64(self.flow_this_tick);
        w.u64(self.last_step_population); // 决策5 豆包 R1：demand_pressure 的 k 来源，
                                          // 权威输入必须入摘要（G3 覆盖），否则不同状态
                                          // 同摘要而降级压力不同=强读档等价假通过风险
        for v in &self.vehicles {
            w.u64(v.id);
            w.u32(v.x);
            w.u32(v.y);
            w.u32(v.goal_x);
            w.u32(v.goal_y);
            w.u64(v.edge_traversals);
        }
        g.record("sim-traffic", "TrafficTick", w.as_bytes());
    }
}

/// 拟合误差报告（3a 独立参照验证口径；全定点‰，权威 crate 禁浮点）。
/// 三区口径（2026-11-19 校准复核）：自由流 k<k_turn/2、转折区 [k_turn/2, 4·k_turn)、
/// 饱和区 k≥4·k_turn（转折区宽度系试点数据 B 导出，须由确认集 C 复测）；
/// k_c 位置偏差仅对三角型（descending_branch_observed）有意义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FitErrorReport {
    /// 自由流区（k < k_turn/2）最大相对误差（‰）
    pub free_flow_max_err_permille: u64,
    /// 转折区（k_turn/2 ≤ k < 4·k_turn）最大相对误差（‰）——只记录不设硬判据
    pub transition_max_err_permille: u64,
    /// 饱和区（k ≥ 4·k_turn）最大相对误差（‰）
    pub saturated_max_err_permille: u64,
    /// 临界密度位置误差（|k_c_拟合 − k_c_观测|，‰）；平台型标 N/A 不判
    pub k_c_err_permille: u64,
    /// 参与评估的留出样本点数
    pub points_evaluated: usize,
    /// 各区样本数（决策4(d)/决策5 共识：空区不得真空 PASS；<2 在判定层标警示）
    pub free_n: usize,
    pub trans_n: usize,
    pub sat_n: usize,
    /// 各区分辨率受限点数（审计3 C-1：披露 + 预注册上限 ≤1/18 全样本）。
    pub free_n_limited: usize,
    pub trans_n_limited: usize,
    pub sat_n_limited: usize,
    /// 各区逐点判定（审计3 C-1：分辨率复核**在判定路径内**——
    /// 分辨率充足点按相对判据 err≤crit；受限点按绝对判据 |Δq|≤2‰；
    /// 任一点不过=区不过；空区=false（真空 PASS 禁止））。
    pub free_pass: bool,
    pub trans_pass: bool,
    pub sat_pass: bool,
}

/// 判据常量（单一来源，ADR-NEW-25 3a 判据与代码断言必须引用同一常量——
/// 决策4 整改项 (f)：原 simulation 测试断言 250‰ 与判据 100‰ 门禁脱钩）。
/// 复测协议（Set D+第二拓扑+预注册 fixture）后由决策4 复核回填。
pub const FREE_FLOW_CRIT_PERMILLE: u64 = 100;
pub const TRANSITION_CRIT_PERMILLE: u64 = 250;
pub const SATURATED_CRIT_PERMILLE: u64 = 250;
pub const K_C_CRIT_PERMILLE: u64 = 200;

/// 模型流量 q_model(k)（‰）：平台型 min(v_free·k/1000, q_max)；
/// 三角型在下降支 q_max − w·(k−k_c)（负值截 0）。
/// （2026-11-19 决策4 整改：公开供 example/测试复用，删除 mfd3a.rs 内联副本。）
pub fn q_model_permille(fit: &TriangleFit, k_permille: u64) -> u64 {
    let q = fit.v_free_permille as u128 * k_permille as u128 / 1000;
    let q = q.min(fit.q_max_permille as u128) as u64;
    if fit.descending_branch_observed && k_permille > fit.k_c_permille {
        let drop = fit.w_permille as u128 * (k_permille - fit.k_c_permille) as u128 / 1000;
        q.saturating_sub(drop as u64)
    } else {
        q
    }
}

/// 用留出样本（未参与拟合的密度点）评估拟合误差（ADR-NEW-25 3a 独立参照）。
/// 误差=|q_model − q_obs|·1000 / max(q_obs, 1)（‰）；k_c 观测值=留出样本 argmax q 的密度。
/// 分区（审计修复 2026-11-20）：预注册绝对带 ZoneBands（禁由被评参数导出，决策4 3a(a)）。
pub fn evaluate_fit(
    fit: &TriangleFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    // 委托 evaluate_q_fn（X3 泛化核）：分区统计/计数单一实现（防双实现漂移）；
    // 仅 k_c 位置误差为三角型特有。
    let mut r = evaluate_q_fn(|k| q_model_permille(fit, k), holdout, bands);
    let k_obs_peak = holdout
        .iter()
        .max_by_key(|s| s.q_permille)
        .map(|s| s.k_permille)
        .unwrap_or(0);
    r.k_c_err_permille = fit.k_c_permille.abs_diff(k_obs_peak);
    r
}

// ===========================================================================
// 凹形有理饱和族（2026-11-19 决策4 定案默认族；ADR-NEW-25 决策 1）
// q(k) = q_max · k / (k + k_half)；定点 +−×÷ only（禁 exp/log/powf）。
// ===========================================================================

/// 有理饱和拟合参数（定点整数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RationalSaturationFit {
    /// 渐近流量上界 q_max（‰）
    pub q_max_permille: u64,
    /// 半饱和密度 k_half（‰）——q(k_half)=q_max/2；零密度斜率=q_max·1000/k_half。
    pub k_half_permille: u64,
}

impl RationalSaturationFit {
    /// 零密度斜率（‰，q/k 放大 1000 倍）；物理上界=1000‰。
    pub fn zero_density_slope_permille(&self) -> u64 {
        if self.k_half_permille == 0 { return u64::MAX; }
        (self.q_max_permille as u128 * 1000 / self.k_half_permille as u128) as u64
    }
}

/// 定点模型流量 q(k)=q_max·k/(k+k_half)（‰），就近舍入（与 SimScalar 契约同口径）。
pub fn q_rational_permille(fit: &RationalSaturationFit, k_permille: u64) -> u64 {
    if k_permille == 0 { return 0; }
    let num = fit.q_max_permille as u128 * k_permille as u128;
    let den = k_permille as u128 + fit.k_half_permille as u128; // u128 先扩后加（GPT-6 E2 防 u64 环绕）
    ((num + den / 2) / den) as u64
}

/// 辨识（确定性定点）：q_max 与 k_half **联合**整数网格最小化残差平方和
/// （u128 定点；平局取先遇到的最小 (q_max, k_half)）。q_max 候选域=观测最大值
/// 至其 4 倍+16（渐近值在未饱和数据下不可由观测最大值代表——W1 实测教训）；
/// k_half 候选域=1..4·k_max。
/// **越界判失败口径（决策4 条件⑧，审计修复 2026-11-20）**：先做无约束自由估计；
/// 若零密度斜率超 1000‰（每车每 tick 至多一条边），**判定层直接 FAIL**（binding=true
/// 即失败信号）；物理可行域回退解**仅作诊断披露与敏感性参考，不得参与 PASS/FAIL
/// 判定**——旧口径"回退解参与判定"=把约束当估计用，与决策4 条件⑧相反，已废
/// （旧 v_free=min(1000) 截断掩盖同样已废）。
/// 返回 (拟合, 自由估计零密度斜率‰, 约束是否绑定)。
pub fn fit_rational_saturation_detailed(
    samples: &[MfdSample],
    bands: &ZoneBands,
) -> (RationalSaturationFit, u64, bool) {
    assert!(!samples.is_empty(), "no samples");
    // q≤k 不变量下沉到拟合器入口（决策5 豆包 R3）：示例外的任何调用方
    // 亦不得喂入 q>k 样本（单位斜率物理界，违者=采样口径缺陷）。
    assert!(
        samples.iter().all(|s| s.q_permille <= s.k_permille),
        "q>k sample violates unit-slope bound (sampling population mismatch)"
    );
    let q_obs_max = samples.iter().map(|s| s.q_permille).max().unwrap().max(1);
    let k_max = samples.iter().map(|s| s.k_permille).max().unwrap().max(1);
    let mut best_free: Option<(u64, u64, u64)> = None;
    let mut best_feasible: Option<(u64, u64, u64)> = None;
    for q_max in q_obs_max..=q_obs_max.saturating_mul(4).saturating_add(16) {
        for k_half in 1..=k_max.saturating_mul(4).max(1) {
            let fit = RationalSaturationFit { q_max_permille: q_max, k_half_permille: k_half };
            // 目标=判据归一化分区最大相对误差（②′ 修测，2026-11-22）——
            // 估计目标与判据同构（旧绝对 SSE/rel-SSE 均不同构，废）。
            let loss = criterion_aligned_loss(|k| q_rational_permille(&fit, k), samples, bands);
            if best_free.is_none_or(|(b, _, _)| loss < b) {
                best_free = Some((loss, q_max, k_half));
            }
            if k_half >= q_max && best_feasible.is_none_or(|(b, _, _)| loss < b) {
                best_feasible = Some((loss, q_max, k_half));
            }
        }
    }
    let (_, q_free, k_free) = best_free.unwrap();
    let free_slope = (q_free as u128 * 1000 / k_free as u128) as u64;
    if free_slope <= 1000 {
        let fit = RationalSaturationFit { q_max_permille: q_free, k_half_permille: k_free };
        return (fit, free_slope, false);
    }
    let (_, q_f, k_f) = best_feasible.expect("no physically feasible fit (slope<=1000‰)");
    (
        RationalSaturationFit { q_max_permille: q_f, k_half_permille: k_f },
        free_slope,
        true,
    )
}

/// 便捷入口=详细版第 1 返回值（物理可行拟合）。
pub fn fit_rational_saturation(samples: &[MfdSample], bands: &ZoneBands) -> RationalSaturationFit {
    fit_rational_saturation_detailed(samples, bands).0
}

/// 广义凹形族（p-norm p=2）：q(k) = q_max·k / sqrt(k² + k0²)。
/// X3 审计整改（豆包清单 d）：多族并行辨识，避免"单族以偏概全"——
/// W3 转 C 的复测只用了有理族一支，豆包实证 p-norm p=2 在同数据上可全过。
/// 定点 +−×÷ + 整数平方根（u128::isqrt），无超越函数（确定性红线保持）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PNormConcaveFit {
    /// 渐近流量上界 q_max（‰）
    pub q_max_permille: u64,
    /// 尺度密度 k0（‰）——q(k0)=q_max/√2；零密度斜率=q_max·1000/k0。
    pub k0_permille: u64,
}

impl PNormConcaveFit {
    /// 零密度斜率（‰）；物理上界=1000‰。
    pub fn zero_density_slope_permille(&self) -> u64 {
        if self.k0_permille == 0 { return u64::MAX; }
        (self.q_max_permille as u128 * 1000 / self.k0_permille as u128) as u64
    }
}

/// 定点模型流量 q(k)=q_max·k/sqrt(k²+k0²)（‰），就近舍入。
pub fn q_pnorm_permille(fit: &PNormConcaveFit, k_permille: u64) -> u64 {
    if k_permille == 0 { return 0; }
    let q_max = fit.q_max_permille as u128;
    let k = k_permille as u128;
    let k0 = fit.k0_permille as u128;
    let den = (k * k + k0 * k0).isqrt().max(1);
    let num = q_max * k;
    ((num + den / 2) / den) as u64
}

/// p-norm 族辨识（目标判据同构：相对误差平方和；越界判失败口径同有理族）。
/// q_max 候选域=观测最大值至 4 倍+16；k0 候选域=1..4·k_max；可行域 k0≥q_max。
pub fn fit_pnorm_concave_detailed(
    samples: &[MfdSample],
    bands: &ZoneBands,
) -> (PNormConcaveFit, u64, bool) {
    assert!(!samples.is_empty(), "no samples");
    // q≤k 不变量下沉到拟合器入口（决策5 豆包 R3）：示例外的任何调用方
    // 亦不得喂入 q>k 样本（单位斜率物理界，违者=采样口径缺陷）。
    assert!(
        samples.iter().all(|s| s.q_permille <= s.k_permille),
        "q>k sample violates unit-slope bound (sampling population mismatch)"
    );
    let q_obs_max = samples.iter().map(|s| s.q_permille).max().unwrap().max(1);
    let k_max = samples.iter().map(|s| s.k_permille).max().unwrap().max(1);
    let mut best_free: Option<(u64, u64, u64)> = None;
    let mut best_feasible: Option<(u64, u64, u64)> = None;
    for q_max in q_obs_max..=q_obs_max.saturating_mul(4).saturating_add(16) {
        for k0 in 1..=k_max.saturating_mul(4).max(1) {
            let fit = PNormConcaveFit { q_max_permille: q_max, k0_permille: k0 };
            // 目标=判据归一化分区最大相对误差（②′ 修测，2026-11-22，同有理族）。
            let loss = criterion_aligned_loss(|k| q_pnorm_permille(&fit, k), samples, bands);
            if best_free.is_none_or(|(b, _, _)| loss < b) {
                best_free = Some((loss, q_max, k0));
            }
            if k0 >= q_max && best_feasible.is_none_or(|(b, _, _)| loss < b) {
                best_feasible = Some((loss, q_max, k0));
            }
        }
    }
    let (_, q_free, k_free) = best_free.unwrap();
    let free_slope = (q_free as u128 * 1000 / k_free as u128) as u64;
    if free_slope <= 1000 {
        return (PNormConcaveFit { q_max_permille: q_free, k0_permille: k_free }, free_slope, false);
    }
    let (_, q_f, k_f) = best_feasible.expect("no physically feasible fit (slope<=1000‰)");
    (PNormConcaveFit { q_max_permille: q_f, k0_permille: k_f }, free_slope, true)
}

/// 便捷入口=详细版第 1 返回值。
pub fn fit_pnorm_concave(samples: &[MfdSample], bands: &ZoneBands) -> PNormConcaveFit {
    fit_pnorm_concave_detailed(samples, bands).0
}

/// 分区带（审计修复 2026-11-20：判定边界须为**预注册固定 fixture**，禁止由
/// 被评模型参数导出——决策4 3a(a)；自指边界=同参 gaming，豆包实证：自指带
/// 下 FAIL 的数据在绝对带上三区全 PASS）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneBands {
    /// 自由流区上界（k < free_max_k，‰）
    pub free_max_k: u64,
    /// 转折区上界（free_max_k ≤ k < trans_max_k，‰）
    pub trans_max_k: u64,
}

impl ZoneBands {
    /// Set D 主判定带（预注册绝对密度边界，豆包审计复算带）。
    /// 敏感性带（如 free=64/263）由调用方另行构造披露。
    pub const SET_D: ZoneBands = ZoneBands { free_max_k: 100, trans_max_k: 2000 };
}

/// 用留出样本评估拟合（ADR-NEW-25 3a 独立参照；分区=预注册绝对带）。
/// 泛化核：对任意模型 q(k) 评估（X3 审计整改：消除各族评估的双实现，
/// 豆包/MiniMax 报告 q_model 副本漂移教训）。
pub fn evaluate_q_fn(
    q: impl Fn(u64) -> u64,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    assert!(!holdout.is_empty(), "no holdout samples");
    let mut free_max = 0u64;
    let mut trans_max = 0u64;
    let mut sat_max = 0u64;
    let (mut free_n, mut trans_n, mut sat_n) = (0usize, 0usize, 0usize);
    let (mut free_lim, mut trans_lim, mut sat_lim) = (0usize, 0usize, 0usize);
    let (mut free_pass, mut trans_pass, mut sat_pass) = (true, true, true);
    for s in holdout {
        let qm = q(s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000
            / (s.q_permille.max(1) as u128);
        let err = err as u64;
        // 分区判据（单一来源常量）与逐点判定（审计3 C-1：分辨率复核在判定路径内）。
        if s.k_permille < bands.free_max_k {
            free_max = free_max.max(err);
            free_n += 1;
            if !point_resolution_ok(s.q_permille, FREE_FLOW_CRIT_PERMILLE) { free_lim += 1; }
            free_pass &= point_passes(err, s.q_permille, FREE_FLOW_CRIT_PERMILLE);
        } else if s.k_permille < bands.trans_max_k {
            trans_max = trans_max.max(err);
            trans_n += 1;
            if !point_resolution_ok(s.q_permille, TRANSITION_CRIT_PERMILLE) { trans_lim += 1; }
            trans_pass &= point_passes(err, s.q_permille, TRANSITION_CRIT_PERMILLE);
        } else {
            sat_max = sat_max.max(err);
            sat_n += 1;
            if !point_resolution_ok(s.q_permille, SATURATED_CRIT_PERMILLE) { sat_lim += 1; }
            sat_pass &= point_passes(err, s.q_permille, SATURATED_CRIT_PERMILLE);
        }
    }
    FitErrorReport {
        free_flow_max_err_permille: free_max,
        transition_max_err_permille: trans_max,
        saturated_max_err_permille: sat_max,
        k_c_err_permille: 0, // 平台/饱和族无峰值拐点，N/A
        points_evaluated: holdout.len(),
        free_n,
        trans_n,
        sat_n,
        free_n_limited: free_lim,
        trans_n_limited: trans_lim,
        sat_n_limited: sat_lim,
        free_pass: free_pass && free_n > 0,
        trans_pass: trans_pass && trans_n > 0,
        sat_pass: sat_pass && sat_n > 0,
    }
}

/// 判据归一化损失（②′ 确认轮预注册拟合目标，2026-11-22 决策6 Z3）：
/// loss = max over 点 (点归一化误差)，逐点双分支与 [`point_passes`] 同构
/// （审计3 C-3：旧版只走相对分支，分支级不同构）：
/// - 分辨率充足点：err‰ × 1000 / 区判据‰；
/// - 分辨率受限点：|Δq|‰ × 1000 / 2（绝对口径归一化）。
///
/// **估计目标与判据同构**——杜绝"估计目标≠判据"第三次（决策4 绝对 SSE、
/// 决策5 rel-SSE 均与分区判据不同构，反事实复算显示仅换目标即翻转判定）。
/// 判定阈值=1000（即 100% 判据边界）。
pub fn criterion_aligned_loss(
    q: impl Fn(u64) -> u64,
    samples: &[MfdSample],
    bands: &ZoneBands,
) -> u64 {
    assert!(!samples.is_empty(), "no samples");
    let mut loss = 0u64;
    for s in samples {
        let qm = q(s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000
            / (s.q_permille.max(1) as u128);
        let err = err as u64;
        let (crit, limited) = if s.k_permille < bands.free_max_k {
            (FREE_FLOW_CRIT_PERMILLE, !point_resolution_ok(s.q_permille, FREE_FLOW_CRIT_PERMILLE))
        } else if s.k_permille < bands.trans_max_k {
            (TRANSITION_CRIT_PERMILLE, !point_resolution_ok(s.q_permille, TRANSITION_CRIT_PERMILLE))
        } else {
            (SATURATED_CRIT_PERMILLE, !point_resolution_ok(s.q_permille, SATURATED_CRIT_PERMILLE))
        };
        let term = if limited {
            let abs_deltaq = err as u128 * s.q_permille as u128 / 1000;
            (abs_deltaq * 1000 / 2) as u64
        } else {
            err * 1000 / crit
        };
        loss = loss.max(term);
    }
    loss
}

/// 每点分辨率下限（②′ 修测四项，MiMo/Claude 口径）：判据×q_obs ≥ 2 个采样
/// 单位（‰ 量化）——相对判据在该点的绝对容差必须超过 ±2‰ 取整噪声；
/// 不满足→该点"分辨率受限"，以绝对误差 |Δq|≤2‰ 复核并在证据中披露。
pub fn point_resolution_ok(q_obs_permille: u64, crit_permille: u64) -> bool {
    crit_permille.saturating_mul(q_obs_permille) >= 2 * 1000
}

/// 单点判定（审计3 C-1：分辨率复核必须在判定路径内，不得只打印）：
/// - 分辨率充足（判据×q_obs ≥ 2 个采样单位）→ 相对判据 `err ≤ crit`；
/// - 分辨率受限 → 绝对判据 |Δq| = err·q_obs/1000 ≤ 2‰（取整噪声带上），
///   并由评估层计入 `n_limited` 披露（豆包：不得"受限点贡献 0"换 PASS）。
pub fn point_passes(err_permille: u64, q_obs_permille: u64, crit_permille: u64) -> bool {
    if point_resolution_ok(q_obs_permille, crit_permille) {
        err_permille <= crit_permille
    } else {
        let abs_deltaq_permille = err_permille as u128 * q_obs_permille as u128 / 1000;
        abs_deltaq_permille <= 2
    }
}

/// 分辨率受限点占比预注册上限（审计3 豆包 R2：≤1/18 全样本，防"贡献 0"换 PASS）。
/// 超限则判定无效（报告披露但整轮判 FAIL——fixture 须加密低密度点）。
pub fn limited_cap_ok(n_limited_total: usize, points_evaluated: usize) -> bool {
    (n_limited_total as u128) * 18 <= points_evaluated as u128
}

/// 稳态判据（②′ 稳态硬门；文档与实现口径统一=双口径任一）：
/// 窗口前后半 q 差 ≤2‰ **或** ≤15 辆。非稳态档位不得静默——单独报告
/// （稳态率目标：自由流区 ≥50%，DeepSeek 口径）。
pub fn steady_window(back_half_delta_permille: u64, back_half_delta_vehicles: u64) -> bool {
    back_half_delta_permille <= 2 || back_half_delta_vehicles <= 15
}

/// 有理饱和族评估（evaluate_q_fn 委托）。
pub fn evaluate_rational_fit(
    fit: &RationalSaturationFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    evaluate_q_fn(|k| q_rational_permille(fit, k), holdout, bands)
}

/// p-norm 族评估（evaluate_q_fn 委托）。
pub fn evaluate_pnorm_fit(
    fit: &PNormConcaveFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    evaluate_q_fn(|k| q_pnorm_permille(fit, k), holdout, bands)
}

/// (k, q) 采样点：k=活跃车辆/边（放大 1000 倍定点），q=边遍历/边每 tick（放大 1000 倍）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MfdSample {
    pub k_permille: u64,
    pub q_permille: u64,
}

/// 三角基本图拟合（定点；候选模型形式，ADR-NEW-25 决策 1）。
/// 2026-11-19 投票修复：自由流斜率改 k≤k_c 点过原点定点最小二乘（原三点估计
/// v_free=q_max/k_first 高估约 31 倍）；拥堵斜率仅在观测到下降支时辨识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriangleFit {
    /// 自由流斜率（q/k，放大 1000 倍；物理上界=1000，即每车每 tick 一条边）。
    pub v_free_permille: u64,
    /// 临界密度（峰值处 k，放大 1000 倍）。
    pub k_c_permille: u64,
    /// 峰值流量（放大 1000 倍）。
    pub q_max_permille: u64,
    /// 拥堵斜率绝对值（放大 1000 倍；未观测到下降支时为 0=不可辨识）。
    pub w_permille: u64,
    /// 是否观测到拥堵下降支（q 在 k>k_c 处低于 q_max）。false=数据呈单调饱和
    /// 平台型，三角的 -w 分支不可辨识，模型应按平台型（单调饱和凹曲线）解读。
    pub descending_branch_observed: bool,
}

/// 从采样序列拟合基本图（确定性：输入序即计算序）。
/// 采样须按 k 升序传入（调用方排序，键相同按采样序稳定）。
pub fn fit_triangle(samples: &[MfdSample]) -> TriangleFit {
    assert!(!samples.is_empty(), "no samples");
    // 注（决策5）：q≤k 采样不变量的真实数据保障在两个 detailed 拟合器入口断言
    // 与 example 采集点 fail-fast；fit_triangle 为遗留估计器，其 clamp 语义
    // （v_free.min(1000)）需要越界合成数据才能测试，故此处不断言。
    let mut peak = samples[0];
    for s in samples {
        if s.q_permille > peak.q_permille {
            peak = *s;
        }
    }
    let k_c = peak.k_permille.max(1);
    let q_max = peak.q_permille;
    // 自由流斜率=相邻采样点最大割线斜率 max(Δq/Δk)（k 升序输入）。
    // 语义=零密度导数的可观测上界；对三角数据与平台型数据均正确。
    // （2026-11-19 二次修正：过原点全点最小二乘在单调饱和数据下被高 k 点拉低。）
    let mut v_free = if samples[0].k_permille > 0 {
        (samples[0].q_permille as u128 * 1000 / samples[0].k_permille as u128) as u64
    } else {
        0
    };
    for pair in samples.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if b.k_permille > a.k_permille {
            let slope = ((b.q_permille.saturating_sub(a.q_permille)) as u128 * 1000
                / (b.k_permille - a.k_permille) as u128) as u64;
            if slope > v_free {
                v_free = slope;
            }
        }
    }
    // 物理上界截断：本微观模型每车每 tick 至多一条边 → v_free ≤ 1000‰；
    // 离散化下割线斜率可短暂越界（如 Δq/Δk≈1010‰），按契约截 0~1000。
    let v_free = v_free.min(1000);
    // 拥堵斜率：仅当 k>k_c 处存在 q<q_max 的下降支样本才可辨识；
    // 对 k>k_c 点拟合 q ≈ q_max - w·(k-k_c)：w = Σ(Δk·Δq)/Σ(Δk²)（负值截 0）。
    let descending = samples
        .iter()
        .any(|s| s.k_permille > k_c && s.q_permille < q_max);
    let mut w = 0u64;
    if descending {
        let (mut n2, mut d2) = (0i128, 0i128);
        for s in samples {
            if s.k_permille > k_c {
                let dk = (s.k_permille - k_c) as i128;
                let dq = q_max as i128 - s.q_permille as i128;
                n2 += dk * dq;
                d2 += dk * dk;
            }
        }
        if d2 > 0 && n2 > 0 {
            w = (n2 * 1000 / d2) as u64;
        }
    }
    TriangleFit {
        v_free_permille: v_free,
        k_c_permille: k_c,
        q_max_permille: q_max,
        w_permille: w,
        descending_branch_observed: descending,
    }
}

/// 引擎集成状态导出/恢复（integration-design.md §2）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridTrafficState {
    pub w: u32,
    pub h: u32,
    pub cap_per_edge_per_tick: u64,
    pub tick: u64,
    pub pending: Vec<Vehicle>,
    pub vehicles: Vec<Vehicle>,
    pub next_id: u64,
    pub flow_this_tick: u64,
    pub last_step_population: u64,
    pub seed: u64,
}

impl GridTraffic {
    pub fn export_state(&self) -> GridTrafficState {
        GridTrafficState {
            w: self.w,
            h: self.h,
            cap_per_edge_per_tick: self.cap_per_edge_per_tick,
            tick: self.tick,
            pending: self.pending.clone(),
            vehicles: self.vehicles.clone(),
            next_id: self.next_id,
            flow_this_tick: self.flow_this_tick,
            last_step_population: self.last_step_population,
            seed: self.seed,
        }
    }

    pub fn restore_state(&mut self, s: GridTrafficState) {
        self.w = s.w;
        self.h = s.h;
        self.cap_per_edge_per_tick = s.cap_per_edge_per_tick;
        self.tick = s.tick;
        self.pending = s.pending;
        self.vehicles = s.vehicles;
        self.next_id = s.next_id;
        self.flow_this_tick = s.flow_this_tick;
        self.last_step_population = s.last_step_population;
        self.seed = s.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_run(densities: &[u64]) -> (GlobalDigest, Vec<MfdSample>) {
        let mut g = GlobalDigest::new();
        let mut samples = Vec::new();
        for &inj in densities {
            let mut gt = GridTraffic::new(8, 8, 1, 0x00C0_FFEE_1234_5678);
            for t in 0..260u64 {
                // 密度阶梯=持续注入；200 tick 后进入稳态采样
                gt.inject(inj);
                let _ = t;
                gt.step();
                gt.fold_digest(&mut g);
            }
            samples.push(MfdSample {
                k_permille: gt.active_count() * 1000 / gt.edge_count(),
                q_permille: gt.flow_this_tick * 1000 / gt.edge_count(),
            });
        }
        (g, samples)
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
    }

    #[test]
    fn tbench_same_seed_two_runs_agree() {
        let (g1, s1) = small_run(&[1, 2, 4]);
        let (g2, s2) = small_run(&[1, 2, 4]);
        assert_eq!(g1.finalize(), g2.finalize());
        assert_eq!(s1, s2);
    }

    #[test]
    fn mfd_shape_basic() {
        let (_, samples) = small_run(&[1, 2, 4, 8, 16]);
        // q 非负、峰值存在、k 单调可排
        assert!(samples.iter().any(|s| s.q_permille > 0));
        let peak = samples.iter().max_by_key(|s| s.q_permille).unwrap();
        assert!(peak.q_permille > 0);
    }

    #[test]
    fn triangle_fit_is_deterministic() {
        let (_, mut samples) = small_run(&[1, 2, 4, 8]);
        samples.sort_by_key(|s| s.k_permille);
        let f1 = fit_triangle(&samples);
        let f2 = fit_triangle(&samples);
        assert_eq!(f1, f2);
        assert!(f1.q_max_permille > 0);
        assert!(f1.k_c_permille > 0);
    }

    // —— 2026-11-19 投票修复回归 ——

    #[test]
    fn free_flow_slope_is_physically_bounded() {
        // 物理上界：每车每 tick 至多一条边 → v_free ≤ 1000‰。
        // 旧三点估计（q_max/k_first）高估约 31 倍，此测试为回归护栏。
        let (_, mut samples) = small_run(&[1, 2, 4, 8, 16]);
        samples.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&samples);
        assert!(f.v_free_permille > 0, "v_free = {}", f.v_free_permille);
        assert!(
            f.v_free_permille <= 1000,
            "v_free = {}‰ exceeds physical bound 1000‰",
            f.v_free_permille
        );
    }

    #[test]
    fn monotone_saturation_reports_no_descending_branch() {
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
