//! 正式基准（开源采纳 O1，criterion）：A* 单请求耗时与展开量基线。
//!
//! 运行：cargo bench -p sim-pathfinding --bench astar_bench
//! 报告口径：criterion 中位数/均值 + 环境注记，归档 docs/benchmarks/criterion-*.md。
//! 注：墙钟仅遥测（2.9 口径），确定性指标以展开计数与逐位一致测试为准。
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sim_network::{Edge, Network};
use sim_pathfinding::find_once;

/// 200×200 双向网格（1mm 网格单位，边代价 1000）——与 G5 系列场景同构。
fn grid(w: u64, h: u64) -> (Network, Vec<(i64, i64)>) {
    let mut edges = Vec::new();
    let mut id = 1u64;
    let mut coords = Vec::new();
    for gy in 0..h {
        for gx in 0..w {
            coords.push(((gx * 1000) as i64, (gy * 1000) as i64));
        }
    }
    for gy in 0..h {
        for gx in 0..w {
            let a = gy * w + gx;
            if gx + 1 < w {
                let b = a + 1;
                edges.push(Edge { id, a, b, cost: 1000, geom: vec![] });
                id += 1;
                edges.push(Edge { id, a: b, b: a, cost: 1000, geom: vec![] });
                id += 1;
            }
            if gy + 1 < h {
                let b = a + w;
                edges.push(Edge { id, a, b, cost: 1000, geom: vec![] });
                id += 1;
                edges.push(Edge { id, a: b, b: a, cost: 1000, geom: vec![] });
                id += 1;
            }
        }
    }
    (Network::from_edges(edges), coords)
}

fn bench_astar(c: &mut Criterion) {
    let (net, coords) = grid(200, 200);
    let nodes = 200 * 200;
    let mut g = c.benchmark_group("astar_200x200");
    g.sample_size(10); // 单次 ~ms 级，10 样本保持基准时长可控
    g.bench_function("corner_to_corner", |b| {
        b.iter(|| {
            let out = find_once(black_box(&net), black_box(&coords), 0u64, nodes - 1, nodes);
            black_box(out)
        })
    });
    g.bench_function("mid_range", |b| {
        b.iter(|| {
            let out = find_once(black_box(&net), black_box(&coords), 10_100u64, 30_300u64, nodes);
            black_box(out)
        })
    });
    g.finish();
}

criterion_group!(benches, bench_astar);
criterion_main!(benches);
