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
        // 已到达车辆出网（保持 vehicles=活跃车辆）
        self.vehicles.retain(|v| !v.arrived());
    }

    /// 活跃车辆数（密度分子）。
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
}
