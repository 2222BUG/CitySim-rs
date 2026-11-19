//! 债务 #30（ADR-NEW-24 决策 2 补遗）：A* 细则四组边界测试。
//!
//! 1. reopen 差分——h 一致性前提被刻意打破时，"g 严格更小 reopen"兜底必须
//!    恢复最优（无 reopen 的实现会得到次优 1010）；
//! 2. 等代价多路径前驱裁决——"g 相等不松弛=保留先发现前驱"，结果逐位稳定；
//! 3. 乱序注入——边集乱序/重编号后最优代价不变，同序重建摘要一致；
//! 4. CSR u32 边界——>u16::MAX 边数（300×300 网格 179,400 边）下偏移正确、
//!    路径代价精确（u32::MAX 上限为文档化确定性 panic，无法在测试内构造）；
//! 5. h 一致性不变式——边代价 ≥ 几何曼哈顿长度时 h(u) ≤ c(u,v)+h(v) 成立，
//!    违反下限则不变式被打破（ADR-NEW-24 前提边界书面化）。

use sim_network::{Edge, Network};
use sim_pathfinding::{find_once, FindOutcome};

fn edge(id: u64, a: u64, b: u64, cost: i64, ga: (i64, i64), gb: (i64, i64)) -> Edge {
    Edge { id, a, b, cost, geom: vec![ga, gb] }
}

/// 参考 Dijkstra（正确性基准，允许 O(n²)）。
fn dijkstra(net: &Network, start: u64, goal: u64) -> Option<i64> {
    use std::collections::BTreeMap;
    let mut dist: BTreeMap<u64, i64> = BTreeMap::new();
    dist.insert(start, 0);
    let mut done: BTreeMap<u64, bool> = BTreeMap::new();
    loop {
        // 选未确定的最小 dist
        let mut best: Option<(u64, i64)> = None;
        for (&n, &d) in &dist {
            if !done.get(&n).copied().unwrap_or(false) && best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((n, d));
            }
        }
        let (u, du) = best?;
        if u == goal {
            return Some(du);
        }
        done.insert(u, true);
        for (_eid, to, c) in net.neighbors(u) {
            let nd = du.saturating_add(c);
            if dist.get(&to).map(|&d| nd < d).unwrap_or(true) {
                dist.insert(to, nd);
            }
        }
    }
}

fn h(coords: &[(i64, i64)], node: u64, goal: u64) -> i64 {
    let (x1, y1) = coords[node as usize];
    let (x2, y2) = coords[goal as usize];
    (x1 - x2).abs().saturating_add((y1 - y2).abs())
}

/// 1. reopen 差分：0→1(1000)/0→2(1)/2→1(2)/1→3(10)；最优=0→2→1→3（13）。
///    B→1 边代价 2 < 几何长度 1005（**刻意违反** ADR-NEW-24 代价下限前提），
///    h 高估 → 节点 1 先以 g=1000 展开闭合，后被 g=3 重开——无 reopen 会返回 1010。
#[test]
fn reopen_recovers_optimal_when_heuristic_inconsistent() {
    let coords = [(0i64, 1000), (0, 0), (0, 1005), (0, 0)];
    let edges = vec![
        edge(0, 0, 1, 1000, (0, 1000), (0, 0)),
        edge(1, 0, 2, 1, (0, 1000), (0, 1005)),
        edge(2, 2, 1, 2, (0, 1005), (0, 0)),
        edge(3, 1, 3, 10, (0, 0), (0, 0)),
    ];
    let net = Network::from_edges(edges);
    assert_eq!(dijkstra(&net, 0, 3), Some(13), "Dijkstra 基准");
    match find_once(&net, &coords, 0, 3, u64::MAX) {
        FindOutcome::Complete { path, cost, .. } => {
            assert_eq!(cost, 13, "reopen 兜底必须恢复最优（无 reopen=1010）");
            assert_eq!(path, vec![1, 2, 3], "路径=0→2→1→3 的边序列");
        }
        other => panic!("expected Complete, got {:?}", std::mem::discriminant(&other)),
    }
}

/// 2. 等代价多路径：0→1→3 与 0→2→3 同代价；"g 相等不松弛=保留先发现前驱"，
///    先发现=邻居展开序（CSR (node,edge) 升序）→ 节点 1 先松弛 → 路径恒 [e0,e2]。
#[test]
fn equal_cost_paths_keep_first_discovered_predecessor() {
    let coords = [(0i64, 0), (0, 1), (1, 0), (1, 1)];
    let edges = vec![
        edge(0, 0, 1, 1, (0, 0), (0, 1)),
        edge(1, 0, 2, 1, (0, 0), (1, 0)),
        edge(2, 1, 3, 1, (0, 1), (1, 1)),
        edge(3, 2, 3, 1, (1, 0), (1, 1)),
    ];
    let net = Network::from_edges(edges);
    let run = || match find_once(&net, &coords, 0, 3, u64::MAX) {
        FindOutcome::Complete { path, cost, .. } => (path, cost),
        _ => panic!("expected Complete"),
    };
    let (p1, c1) = run();
    let (p2, c2) = run();
    assert_eq!(c1, 2);
    assert_eq!(p1, vec![0, 2], "先发现前驱（node 1 小序先松弛）保留");
    assert_eq!((p1, c1), (p2, c2), "逐位稳定");
}

/// 3. 乱序注入：同构边集乱序+按插入序重编号 → 最优代价不变；同序重建 → 摘要一致。
#[test]
fn shuffled_edge_insertion_preserves_optimal_cost_and_determinism() {
    // 5×5 网格，代价=几何长度（h 一致）
    let mut edges = Vec::new();
    let mut coords = vec![(0i64, 0); 25];
    for i in 0..5u64 {
        for j in 0..5u64 {
            coords[(i * 5 + j) as usize] = (j as i64 * 1000, i as i64 * 1000);
        }
    }
    let mut id = 0u64;
    for i in 0..5u64 {
        for j in 0..5u64 {
            let n = i * 5 + j;
            let p = coords[n as usize];
            if j + 1 < 5 {
                let m = i * 5 + j + 1;
                edges.push(edge(id, n, m, 1000, p, coords[m as usize]));
                id += 1;
            }
            if i + 1 < 5 {
                let m = (i + 1) * 5 + j;
                edges.push(edge(id, n, m, 1000, p, coords[m as usize]));
                id += 1;
            }
        }
    }
    // 确定性乱序（无 RNG 依赖）：反转 + 交错
    let mut shuffled: Vec<Edge> = edges.iter().rev().cloned().collect();
    for (i, e) in shuffled.iter_mut().enumerate() {
        e.id = i as u64; // 按插入序重编号
    }
    let net_canon = Network::from_edges(edges);
    let net_shuf = Network::from_edges(shuffled.clone());
    let net_shuf2 = Network::from_edges(shuffled);
    let cost_of = |net: &Network| match find_once(net, &coords, 0, 24, u64::MAX) {
        FindOutcome::Complete { cost, .. } => cost,
        _ => panic!("expected Complete"),
    };
    let c_canon = cost_of(&net_canon);
    assert_eq!(c_canon, 8000, "角到角 8 边 × 1000");
    assert_eq!(cost_of(&net_shuf), c_canon, "乱序/重编号不改变最优代价");
    assert_eq!(cost_of(&net_shuf2), c_canon, "逐位稳定");
    assert_eq!(net_shuf.digest(), net_shuf2.digest(), "同序重建摘要一致");
    assert_eq!(net_canon.digest(), Network::from_edges(net_canon.edges().cloned().collect()).digest());
}

/// 4. CSR u32 边界：300×300 网格（90,000 节点 / 179,400 边 > u16::MAX）下偏移
///    正确、代价精确。u32::MAX 边数上限=文档化确定性 panic（无法在测试构造）。
#[test]
fn csr_offsets_survive_beyond_u16_edge_count() {
    let w = 300u64;
    let mut edges = Vec::new();
    let mut id = 0u64;
    for i in 0..w {
        for j in 0..w {
            let n = i * w + j;
            if j + 1 < w {
                edges.push(edge(id, n, n + 1, 1000, (0, 0), (0, 0)));
                id += 1;
            }
            if i + 1 < w {
                edges.push(edge(id, n, n + w, 1000, (0, 0), (0, 0)));
                id += 1;
            }
        }
    }
    assert!(edges.len() > u16::MAX as usize, "必须超过 u16 边数以验证 u32 偏移");
    let n_edges = edges.len();
    let net = Network::from_edges(edges);
    assert_eq!(net.node_count(), w * w);
    // 对角：598 条边 × 1000（坐标用 (0,0) 占位，h 恒 0 → Dijkstra 语义，代价仍精确）
    match find_once(&net, &[(0i64, 0); 1], 0, w * w - 1, u64::MAX) {
        FindOutcome::Complete { cost, .. } => assert_eq!(cost, 598 * 1000, "边数 {n_edges}"),
        _ => panic!("expected Complete"),
    }
}

/// 5. h 一致性不变式：代价 ≥ 几何曼哈顿长度 → h(u) ≤ c(u,v)+h(v)；
///    违反下限 → 不变式被打破（前提边界书面化）。
#[test]
fn h_consistency_invariant_tracks_cost_lower_bound() {
    let coords = [(0i64, 0), (1000, 0), (1000, 1000)];
    // 合规：cost ≥ 几何长度
    for e in [
        edge(0, 0, 1, 1000, (0, 0), (1000, 0)),
        edge(1, 1, 2, 1000, (1000, 0), (1000, 1000)),
    ] {
        let lhs = h(&coords, e.a, 2);
        let rhs = e.cost.saturating_add(h(&coords, e.b, 2));
        assert!(lhs <= rhs, "h 一致性必须成立: {} <= {}", lhs, rhs);
    }
    // 违例：cost 1 < 几何长度 1000
    let e = edge(2, 0, 1, 1, (0, 0), (1000, 0));
    let lhs = h(&coords, e.a, 2);
    let rhs = e.cost.saturating_add(h(&coords, e.b, 2));
    assert!(lhs > rhs, "违反代价下限前提时 h 一致性必然被打破（reopen 兜底的适用面）");
}
