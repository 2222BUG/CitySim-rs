//! proptest 属性测试（开源采纳 O1）：sim-network 两条不变量。
//! 失败输入自动落 `proptest-regressions/`（提交入库=可复现回放）。
//!
//! - 属性 1 发布原子性：非法事务（重复边 id）零改动（version/digest 均不变）。
//! - 属性 2 过期合法性方向：路径上任一边被删 ⇒ stale_check 必返回 Recompute
//!   （Accept 不可能踩在丢边上——ADR-NEW-23 条文 9 的机器可检形式）。
use proptest::prelude::*;
use sim_network::{Edge, Network, Op, StaleOutcome, Tx};

/// 2×2 双向小网格：节点 0..4，边 id 1..=8（两两成对：正向奇数/反向偶数）。
fn small_net() -> Network {
    let mut edges = Vec::new();
    let mut id = 1u64;
    let mut pair = |edges: &mut Vec<Edge>, id: &mut u64, a: u64, b: u64, geom: Vec<(i64, i64)>| {
        edges.push(Edge { id: *id, a, b, cost: 1000, geom: geom.clone() });
        *id += 1;
        edges.push(Edge { id: *id, a: b, b: a, cost: 1000, geom });
        *id += 1;
    };
    pair(&mut edges, &mut id, 0, 1, vec![(0, 0), (500, 0)]);
    pair(&mut edges, &mut id, 2, 3, vec![(0, 1000), (500, 1000)]);
    pair(&mut edges, &mut id, 0, 2, vec![(0, 0), (0, 1000)]);
    pair(&mut edges, &mut id, 1, 3, vec![(500, 0), (500, 1000)]);
    Network::from_edges(edges)
}

proptest! {
    /// 非法事务（重复边 id）发布失败 ⇒ 状态零改动。
    #[test]
    fn failed_publish_never_mutates(dup_id in 1u64..=8, tx_id in 1u64..1000) {
        let mut net = small_net();
        let v0 = net.version();
        let d0 = net.digest();
        let tx = Tx {
            id: tx_id,
            ops: vec![
                Op::AddEdge(Edge { id: dup_id, a: 0, b: 1, cost: 1, geom: vec![(0, 0), (100, 0)] }),
                Op::AddEdge(Edge { id: dup_id, a: 2, b: 3, cost: 1, geom: vec![(0, 900), (100, 900)] }),
            ],
        };
        let _ = net.publish(tx); // DuplicateEdge 语义：整体拒绝
        prop_assert_eq!(net.version(), v0, "failed publish must not bump version");
        prop_assert_eq!(net.digest(), d0, "failed publish must not change digest");
    }

    /// 路径上任一边被拓扑事务删除 ⇒ stale_check 必为 Recompute（合法性方向）。
    #[test]
    fn removed_path_edge_is_never_accepted(path_edge_idx in 0usize..2, tx_id in 1u64..1000) {
        let mut net = small_net();
        let base = net.version();
        // 路径=边 1→2（水平正向对的两条），选一条删除
        let path_edges = [1u64, 2u64];
        let removed = path_edges[path_edge_idx];
        let tx = Tx { id: tx_id, ops: vec![Op::RemoveEdge { id: removed }] };
        let _ = net.publish(tx);
        match net.stale_check(&path_edges, base) {
            StaleOutcome::Accept => panic!("Accept on a removed path edge violates legality"),
            StaleOutcome::Recompute { .. } => {}
        }
    }
}
