//! T-Bench-1：MFD 交通基准场景（ADR-NEW-25 决策 4，Stage 2C C3b）。
//! 40×40 双向网格、密度阶梯持续注入、200 tick 稳态后采样 (k, q)。
//! 运行：cargo run --release --example tbench1 -p sim-traffic

use sim_traffic::{fit_triangle, GridTraffic, MfdSample};

fn main() {
    const W: u32 = 40;
    const H: u32 = 40;
    const SEED: u64 = 0x00C0_FFEE_1234_5678;
    const TICKS: u64 = 260;
    let densities: [u64; 8] = [2, 4, 8, 16, 32, 64, 128, 256];

    println!("T-Bench-1 (ADR-NEW-25) — 40x40 grid, seed {SEED:#x}, {TICKS} ticks/density");
    println!("density_inj | active_veh | k(‰edges) | q(‰edges/tick)");
    let mut samples: Vec<MfdSample> = Vec::new();
    for &inj in &densities {
        let mut gt = GridTraffic::new(W, H, 1, SEED);
        for _ in 0..TICKS {
            gt.inject(inj);
            gt.step();
        }
        let s = MfdSample {
            k_permille: gt.active_count() * 1000 / gt.edge_count(),
            q_permille: gt.flow_this_tick * 1000 / gt.edge_count(),
        };
        samples.push(s);
        println!("{:>10} | {:>10} | {:>9} | {:>13}", inj, gt.active_count(), s.k_permille, s.q_permille);
    }

    // 按 k 升序排序（确定性：稳定排序）
    let mut sorted = samples.clone();
    sorted.sort_by_key(|s| s.k_permille);
    let fit = fit_triangle(&sorted);
    println!("\ntriangle fit (fixed-point permille):");
    println!("  v_free = {}", fit.v_free_permille);
    println!("  k_c    = {}", fit.k_c_permille);
    println!("  q_max  = {}", fit.q_max_permille);
    println!("  w      = {}", fit.w_permille);
    println!("\nnote: thresholds in ADR-NEW-25 are placeholders pending calibration review;");
    println!("      MFD output is non-authoritative statistics/degradation input (ADR-NEW-25 决策 2).");
}
