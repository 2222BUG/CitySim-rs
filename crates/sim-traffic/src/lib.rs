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
    pub fn fold_digest(&self, g: &mut GlobalDigest) {
        let mut w = CanonicalWriter::new("sim-traffic", "TrafficTick");
        w.u64(self.tick);
        w.u64(self.vehicles.len() as u64);
        w.u64(self.flow_this_tick);
        for v in &self.vehicles {
            w.u64(v.id);
            w.u32(v.x);
            w.u32(v.y);
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriangleFit {
    /// 自由流斜率（q/k，放大 1000 倍）。
    pub v_free_permille: u64,
    /// 临界密度（峰值处 k，放大 1000 倍）。
    pub k_c_permille: u64,
    /// 峰值流量（放大 1000 倍）。
    pub q_max_permille: u64,
    /// 拥堵斜率绝对值（放大 1000 倍）。
    pub w_permille: u64,
}

/// 从采样序列拟合三角基本图（确定性：输入序即计算序）。
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
    // 自由流斜率：取 k 最小的非零点（k>0）
    let first = samples.iter().find(|s| s.k_permille > 0).copied().unwrap_or(peak);
    let v_free = (q_max * 1000).checked_div(first.k_permille.max(1)).unwrap_or(0);
    // 拥堵斜率：取 k 最大的点
    let last = samples.iter().max_by_key(|s| s.k_permille).copied().unwrap_or(peak);
    let w = if last.k_permille > k_c {
        ((q_max.saturating_sub(last.q_permille)) * 1000)
            .checked_div(last.k_permille - k_c)
            .unwrap_or(0)
    } else {
        0
    };
    TriangleFit {
        v_free_permille: v_free,
        k_c_permille: k_c,
        q_max_permille: q_max,
        w_permille: w,
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
}
