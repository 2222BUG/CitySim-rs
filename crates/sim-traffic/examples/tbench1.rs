//! T-Bench-1：MFD 交通基准场景（ADR-NEW-25 决策 4，Stage 2C C3b）。
//! 40×40 双向网格、密度阶梯持续注入；2026-11-19 投票修复：
//! - 采样口径=窗口均值（末 60 tick 均值）+ 稳态判据（窗口前后半 q 差 ≤2‰）
//! - 双向扫描（升序+降序）滞回检测（同注入点 |q_up−q_down| ≤2‰ 视为无滞回）
//! - 拟合输出含下降支辨识标志（平台型 vs 三角）
//!
//! 运行：cargo run --release --example tbench1 -p sim-traffic

use sim_traffic::{fit_triangle, GridTraffic, MfdSample};

const W: u32 = 40;
const H: u32 = 40;
const SEED: u64 = 0x00C0_FFEE_1234_5678;
const TICKS: u64 = 260;
const WINDOW: u64 = 60;
const STEADY_TOL_PERMILLE: u64 = 2;
const DENSITIES_UP: [u64; 8] = [2, 4, 8, 16, 32, 64, 128, 256];

struct Sample {
    inj: u64,
    k_mean: u64,
    q_mean: u64,
    q_min: u64,
    q_max: u64,
    steady: bool,
}

fn run_density(inj: u64) -> Sample {
    let mut gt = GridTraffic::new(W, H, 1, SEED);
    let mut k_sum = 0u64;
    let mut q_sum = 0u64;
    let mut q_min = u64::MAX;
    let mut q_max = 0u64;
    let mut q_first_half = 0u64;
    let mut q_second_half = 0u64;
    for t in 0..TICKS {
        gt.inject(inj);
        gt.step();
        if t >= TICKS - WINDOW {
            let k = gt.last_step_population * 1000 / gt.edge_count();
            let q = gt.flow_this_tick * 1000 / gt.edge_count();
            k_sum += k;
            q_sum += q;
            q_min = q_min.min(q);
            q_max = q_max.max(q);
            if t >= TICKS - WINDOW / 2 {
                q_second_half += q;
            } else {
                q_first_half += q;
            }
        }
    }
    let steady = {
        let fh = q_first_half / (WINDOW / 2);
        let sh = q_second_half / (WINDOW / 2);
        fh.abs_diff(sh) <= STEADY_TOL_PERMILLE
    };
    Sample {
        inj,
        k_mean: k_sum / WINDOW,
        q_mean: q_sum / WINDOW,
        q_min,
        q_max,
        steady,
    }
}

fn main() {
    println!("T-Bench-1 (ADR-NEW-25) — 40x40 grid, seed {SEED:#x}, {TICKS} ticks/density");
    println!("sampling: window mean over last {WINDOW} ticks (2026-11-19 vote fix; steady = front/back half q diff <= {STEADY_TOL_PERMILLE}‰)");
    println!("inj | k_mean(‰) | q_mean(‰) | q_min | q_max | steady");

    let mut up: Vec<Sample> = Vec::new();
    for &inj in &DENSITIES_UP {
        let s = run_density(inj);
        println!("{:>3} | {:>9} | {:>9} | {:>5} | {:>5} | {}", s.inj, s.k_mean, s.q_mean, s.q_min, s.q_max, s.steady);
        up.push(s);
    }

    // 降序扫描（滞回检测）
    println!("\ndescending sweep (hysteresis check):");
    println!("inj | q_mean_dn(‰) | q_mean_up(‰) | diff(‰) | hysteresis");
    let mut down: Vec<Sample> = Vec::new();
    let mut hysteresis = false;
    for &inj in DENSITIES_UP.iter().rev() {
        let s = run_density(inj);
        let up_q = up.iter().find(|u| u.inj == inj).map(|u| u.q_mean).unwrap_or(0);
        let diff = s.q_mean.abs_diff(up_q);
        if diff > STEADY_TOL_PERMILLE {
            hysteresis = true;
        }
        println!("{:>3} | {:>12} | {:>12} | {:>8} | {}", s.inj, s.q_mean, up_q, diff, diff > STEADY_TOL_PERMILLE);
        down.push(s);
    }
    println!("\nhysteresis observed: {hysteresis}");

    // 拟合（升序扫描样本，按 k 升序）
    let mut samples: Vec<MfdSample> = up.iter().map(|s| MfdSample { k_permille: s.k_mean, q_permille: s.q_mean }).collect();
    samples.sort_by_key(|s| s.k_permille);
    let fit = fit_triangle(&samples);
    println!("\nbasic-diagram fit (fixed-point permille):");
    println!("  v_free                    = {}", fit.v_free_permille);
    println!("  k_c                       = {}", fit.k_c_permille);
    println!("  q_max                     = {}", fit.q_max_permille);
    println!("  w                         = {}", fit.w_permille);
    println!("  descending_branch_observed = {}", fit.descending_branch_observed);
    if !fit.descending_branch_observed {
        println!("  interpretation: monotone-saturation (plateau) — congested branch not identifiable;");
        println!("  triangle -w is not supported by this data (point-queue model has no spillback).");
    }
    println!("\nnote: thresholds in ADR-NEW-25 are placeholders pending calibration review;");
    println!("      MFD output is non-authoritative statistics/degradation input (ADR-NEW-25 决策 2).");
}
