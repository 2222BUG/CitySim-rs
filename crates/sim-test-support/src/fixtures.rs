//! 测试数据和场景（EC-07 fixtures）。
//!
//! 全部确定性：无墙钟、无随机，场景即参数。网格单位与主文档一致
//! （1 mm 整数网格，见 ADR-NEW-23；坐标范围见 2.10 参数基线）。

/// 场景参数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixture {
    pub name: &'static str,
    pub seed: u64,
    pub agent_count: u64,
    pub tick_count: u64,
    pub grid_nodes_x: u64,
    pub grid_nodes_y: u64,
    pub spacing_mm: u64,
}

impl Fixture {
    /// 参考场景（2.10 已定案）：100,000 Agent × 1,000 Tick，200×200 网格。
    pub fn reference() -> Fixture {
        Fixture {
            name: "reference-100k",
            seed: 0xC5_17_5EED,
            agent_count: 100_000,
            tick_count: 1_000,
            grid_nodes_x: 200,
            grid_nodes_y: 200,
            spacing_mm: 1_000,
        }
    }

    /// R2 门禁场景（2.10 已定案）：10,000 Agent × 1,000 Tick，200×200 网格。
    pub fn r2_gate() -> Fixture {
        Fixture {
            name: "r2-gate-10k",
            seed: 0x2026_1118,
            agent_count: 10_000,
            tick_count: 1_000,
            grid_nodes_x: 200,
            grid_nodes_y: 200,
            spacing_mm: 1_000,
        }
    }

    /// 冒烟场景（PR 过渡期缩小场景）：1,000 Agent × 200 Tick。
    pub fn smoke() -> Fixture {
        Fixture {
            name: "smoke-1k",
            seed: 0x5A17_E,
            agent_count: 1_000,
            tick_count: 200,
            grid_nodes_x: 32,
            grid_nodes_y: 32,
            spacing_mm: 1_000,
        }
    }
}

/// 确定性网格路网边表：(from, to, cost_mm)。行优先遍历，输出即稳定序
/// （2.9 条 4：不得依赖容器迭代顺序，边表应按稳定键排序）。
///
/// 边集合：每个节点向右、向下各连一条（边界除外），cost = spacing_mm。
pub fn grid_edges(f: &Fixture) -> Vec<(u64, u64, u64)> {
    let mut edges = Vec::new();
    let nx = f.grid_nodes_x;
    let ny = f.grid_nodes_y;
    for y in 0..ny {
        for x in 0..nx {
            let id = y * nx + x;
            if x + 1 < nx {
                edges.push((id, id + 1, f.spacing_mm)); // 向右
            }
            if y + 1 < ny {
                edges.push((id, id + nx, f.spacing_mm)); // 向下
            }
        }
    }
    edges
}

/// 节点坐标（1 mm 网格）：(x_mm, y_mm)。
pub fn node_coord(f: &Fixture, node: u64) -> (u64, u64) {
    (node % f.grid_nodes_x * f.spacing_mm, node / f.grid_nodes_x * f.spacing_mm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_edges_count_and_stability() {
        let f = Fixture::smoke();
        let e1 = grid_edges(&f);
        let e2 = grid_edges(&f);
        // 32×32 网格：水平 32*31 + 垂直 31*32 = 1984
        assert_eq!(e1.len(), 1984);
        assert_eq!(e1, e2, "edge order must be deterministic");
        // 稳定序：按 (from, to) 严格递增
        for w in e1.windows(2) {
            assert!(w[0].0 < w[1].0 || (w[0].0 == w[1].0 && w[0].1 < w[1].1));
        }
    }

    #[test]
    fn node_coord_matches_spacing() {
        let f = Fixture::smoke();
        assert_eq!(node_coord(&f, 0), (0, 0));
        assert_eq!(node_coord(&f, 33), (1_000, 1_000));
    }
}
