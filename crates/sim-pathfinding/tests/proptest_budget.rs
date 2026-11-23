//! proptest 属性测试（开源采纳 O1）：A* 预算上界不变量。
//! 失败输入自动落 `proptest-regressions/`（提交入库=可复现回放）。
//!
//! - 属性：任意请求、任意预算下，find_once 的展开计数不超过授予预算
//!   （TickBudget 扣减依据=expanded，超预算是不可能的——ADR-NEW-24 条文 3 的机器可检形式）。
use proptest::prelude::*;
use sim_network::{Edge, Network};
use sim_pathfinding::{find_once, FindOutcome};

fn small_grid() -> (Network, Vec<(i64, i64)>) {
    let w = 4u64;
    let mut edges = Vec::new();
    let mut id = 1u64;
    let mut coords = Vec::new();
    for gy in 0..w {
        for gx in 0..w {
            coords.push(((gx * 1000) as i64, (gy * 1000) as i64));
        }
    }
    for gy in 0..w {
        for gx in 0..w {
            let a = gy * w + gx;
            if gx + 1 < w {
                let b = a + 1;
                edges.push(Edge { id, a, b, cost: 1000, geom: vec![] });
                id += 1;
                edges.push(Edge { id, a: b, b: a, cost: 1000, geom: vec![] });
                id += 1;
            }
            if gy + 1 < w {
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

proptest! {
    /// 展开计数永不超过授予预算。
    #[test]
    fn expansions_never_exceed_budget(
        start in 0u64..16,
        goal in 0u64..16,
        budget in 0u64..128,
    ) {
        let (net, coords) = small_grid();
        let out = find_once(&net, &coords, start, goal, budget);
        prop_assert!(
            out.expanded() <= budget,
            "expanded {} > budget {} (outcome {:?})",
            out.expanded(),
            budget,
            std::mem::discriminant(&out)
        );
    }
}
