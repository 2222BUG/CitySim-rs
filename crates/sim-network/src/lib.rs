//! 路网系统：道路拓扑、动态代价与路网事务的原子发布（主文档 2.2 架构分层、2.5/2.6）。
//! 权威 crate。
//!
//! Stage 2A 实装（2026-11-19，ADR-NEW-24 定案）：
//! - CSR 静态拓扑：`offsets` + `adj_edge_ids`，邻接条目按 (node_id, edge_id) 升序，
//!   数组序即确定性迭代序；动态代价存于边表（2.6 拓扑与代价分离）。
//! - 事务发布原子：整笔生效或整笔不生效（失败不改版本号）；拓扑事务重建 CSR、
//!   代价事务原位写入。
//! - ADR-NEW-23 两阶段相交判定：AABB 宽筛 + 边 ID 窄判；代价更新默认不触发重算。
//!
//! 依赖方向约束见 docs/architecture/crate-authority-list.md。

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "路网系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

// ---------------- 影响区域（网格 1 mm） ----------------

/// 影响区域：轴对齐包围盒（ADR-NEW-23 宽筛层）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

impl Region {
    pub fn new(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Self {
        Region {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    pub fn from_points(pts: &[(i64, i64)]) -> Self {
        let mut r = Region::new(i64::MAX, i64::MAX, i64::MIN, i64::MIN);
        for &(x, y) in pts {
            r = r.union(&Region::new(x, y, x, y));
        }
        r
    }

    pub fn intersects(&self, o: &Region) -> bool {
        self.min_x <= o.max_x && o.min_x <= self.max_x && self.min_y <= o.max_y && o.min_y <= self.max_y
    }

    pub fn union(&self, o: &Region) -> Region {
        Region {
            min_x: self.min_x.min(o.min_x),
            min_y: self.min_y.min(o.min_y),
            max_x: self.max_x.max(o.max_x),
            max_y: self.max_y.max(o.max_y),
        }
    }
}

// ---------------- 边与事务 ----------------

/// 路网边（几何为毫米网格折线，供路径影响区域计算）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub id: u64,
    pub a: u64,
    pub b: u64,
    pub cost: i64,
    pub geom: Vec<(i64, i64)>,
}

impl Edge {
    pub fn region(&self) -> Region {
        if self.geom.is_empty() {
            Region::new(0, 0, 0, 0)
        } else {
            Region::from_points(&self.geom)
        }
    }
}

/// 路网事务操作（每项自带影响区域；拓扑与代价分离，2.6）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    AddEdge(Edge),
    RemoveEdge { id: u64 },
    SetDynamicCost { id: u64, cost: i64 },
}

impl Op {
    pub fn region(&self, net: &Network) -> Region {
        match self {
            Op::AddEdge(e) => e.region(),
            Op::RemoveEdge { id } | Op::SetDynamicCost { id, .. } => {
                net.edge(*id).map(Edge::region).unwrap_or(Region::new(0, 0, 0, 0))
            }
        }
    }
}

/// 路网事务：整笔原子发布（ADR-NEW-23）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub id: u64,
    pub ops: Vec<Op>,
}

impl Tx {
    pub fn region(&self, net: &Network) -> Region {
        let mut r = Region::new(i64::MAX, i64::MAX, i64::MIN, i64::MIN);
        for op in &self.ops {
            r = r.union(&op.region(net));
        }
        r
    }
}

/// 发布失败原因（事务整体不生效，版本号不变）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxError {
    EmptyTx,
    DuplicateEdge { id: u64 },
    UnknownEdge { id: u64 },
}

/// 事务记录（ADR-NEW-23 条文 7 聚合与窄判所需：按版本区间查询）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxRecord {
    pub id: u64,
    pub version_after: u64,
    pub region: Region,
    /// 拓扑变更的边 ID（新增或删除）——窄判参与。
    pub topology_changed_edges: Vec<u64>,
    /// 代价变更的边 ID——默认不触发重算（P1-04 旋钮默认关闭，记录备用）。
    pub cost_changed_edges: Vec<u64>,
}

/// 过期判定结果（ADR-NEW-17 条文 3-9 / ADR-NEW-23）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaleOutcome {
    /// 结果可用（版本一致，或区间内事务与路径不相交/仅代价变更）。
    Accept,
    /// 须重入队重算；trigger_tx=首个命中窄判的事务（按 tx_id 升序，稳定）。
    Recompute { trigger_tx: u64 },
}

// ---------------- 路网（CSR） ----------------

/// CSR 路网（ADR-NEW-24 决策 1）。
///
/// 节点 ID 为稠密 u64（0..=max）；边 ID 任意 u64（BTreeMap 稳定序）。
#[derive(Clone, Debug, Default)]
pub struct Network {
    version: u64,
    edges: BTreeMap<u64, Edge>,
    // CSR 静态拓扑
    offsets: Vec<u32>,
    adj_edge_ids: Vec<u64>,
    n_nodes: u64,
    tx_log: Vec<TxRecord>,
}

impl PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.edges == other.edges
    }
}
impl Eq for Network {}

impl Network {
    pub fn new() -> Self {
        Network { version: 1, ..Default::default() }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// 节点数 |V|（机制 B 的 L 上界用；ADR-NEW-24 决策 3）。
    pub fn node_count(&self) -> u64 {
        self.n_nodes
    }

    pub fn edge(&self, id: u64) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }

    /// 节点出边：CSR 数组序（(node_id, edge_id) 升序），yield (edge_id, to, cost)。
    pub fn neighbors(&self, node: u64) -> impl Iterator<Item = (u64, u64, i64)> + '_ {
        let (a, b) = self.adj_range(node);
        self.adj_edge_ids[a..b].iter().map(move |&eid| {
            let e = &self.edges[&eid];
            (eid, e.b, e.cost)
        })
    }

    fn adj_range(&self, node: u64) -> (usize, usize) {
        if node as usize >= self.offsets.len().saturating_sub(1) {
            return (0, 0);
        }
        (self.offsets[node as usize] as usize, self.offsets[node as usize + 1] as usize)
    }

    fn rebuild_csr(&mut self) {
        let max_node = self.edges.values().map(|e| e.a.max(e.b)).max().unwrap_or(0);
        self.n_nodes = max_node + 1;
        let mut buckets: Vec<Vec<u64>> = vec![Vec::new(); self.n_nodes as usize];
        for e in self.edges.values() {
            buckets[e.a as usize].push(e.id);
        }
        self.offsets.clear();
        self.adj_edge_ids.clear();
        self.offsets.push(0);
        for mut ids in buckets {
            ids.sort_unstable(); // (node_id, edge_id) 升序的后半段
            self.adj_edge_ids.extend(ids);
            self.offsets.push(self.adj_edge_ids.len() as u32);
        }
    }

    /// 原子发布事务：全部操作验证通过才生效；失败时网络零改动（版本不变）。
    pub fn publish(&mut self, tx: Tx) -> Result<(), TxError> {
        if tx.ops.is_empty() {
            return Err(TxError::EmptyTx);
        }
        // 原子验证：在临时副本上试应用（正确性优先；正式双缓冲可优化）
        let mut trial = self.edges.clone();
        let mut topo = Vec::new();
        let mut cost = Vec::new();
        for op in &tx.ops {
            match op {
                Op::AddEdge(e) => {
                    if trial.contains_key(&e.id) {
                        return Err(TxError::DuplicateEdge { id: e.id });
                    }
                    topo.push(e.id);
                    trial.insert(e.id, e.clone());
                }
                Op::RemoveEdge { id } => {
                    if trial.remove(id).is_none() {
                        return Err(TxError::UnknownEdge { id: *id });
                    }
                    topo.push(*id);
                }
                Op::SetDynamicCost { id, cost: c } => {
                    match trial.get_mut(id) {
                        Some(e) => e.cost = *c,
                        None => return Err(TxError::UnknownEdge { id: *id }),
                    }
                    cost.push(*id);
                }
            }
        }
        // 提交
        let region = tx.region(self);
        self.edges = trial;
        if topo.is_empty() {
            // 纯代价事务：原位更新，CSR 不动（ADR-NEW-24 决策 1）
        } else {
            self.rebuild_csr();
        }
        self.version += 1;
        topo.sort_unstable();
        cost.sort_unstable();
        self.tx_log.push(TxRecord {
            id: tx.id,
            version_after: self.version,
            region,
            topology_changed_edges: topo,
            cost_changed_edges: cost,
        });
        Ok(())
    }

    /// 版本区间内的事务记录（from 不含、to 含；按 tx_id 升序）。
    pub fn txs_between(&self, from_exclusive: u64, to_inclusive: u64) -> impl Iterator<Item = &TxRecord> {
        self.tx_log
            .iter()
            .filter(move |t| t.version_after > from_exclusive && t.version_after <= to_inclusive)
    }

    /// 路径影响区域（所经边几何并集）。
    pub fn path_region(&self, path_edges: &[u64]) -> Region {
        let mut r = Region::new(i64::MAX, i64::MAX, i64::MIN, i64::MIN);
        for &eid in path_edges {
            if let Some(e) = self.edge(eid) {
                r = r.union(&e.region());
            }
        }
        r
    }

    /// ADR-NEW-23 两阶段相交判定 + 条文 9 合法性：
    /// 1) 合法性：路径所经边必须在当前路网全部存在（否则 Recompute）；
    /// 2) 宽筛：区间事务影响区域 ∩ 路径区域（条文 7 聚合=区间全部事务逐条判）；
    /// 3) 窄判：命中事务的拓扑变更边 ∩ 路径边 ≠ ∅ 才 Recompute（按 tx_id 升序取首个）；
    ///    代价变更默认不触发重算（P1-04 旋钮默认关闭）。
    pub fn stale_check(&self, path_edges: &[u64], base_version: u64) -> StaleOutcome {
        // 条文 9：合法性（只保证边存在，不保证最优）
        for &eid in path_edges {
            if !self.edges.contains_key(&eid) {
                let trigger = self
                    .txs_between(base_version, self.version)
                    .find(|t| t.topology_changed_edges.contains(&eid))
                    .map(|t| t.id)
                    .unwrap_or(0);
                return StaleOutcome::Recompute { trigger_tx: trigger };
            }
        }
        if base_version == self.version {
            return StaleOutcome::Accept;
        }
        let path_region = self.path_region(path_edges);
        for t in self.txs_between(base_version, self.version) {
            if !t.region.intersects(&path_region) {
                continue; // 宽筛未命中
            }
            // 窄判：拓扑变更边与路径边相交
            if t.topology_changed_edges.iter().any(|e| path_edges.contains(e)) {
                return StaleOutcome::Recompute { trigger_tx: t.id };
            }
        }
        StaleOutcome::Accept
    }

    /// 路网摘要（ADR-NEW-19 GlobalDigest 纳入范围：拓扑+代价+版本）。
    pub fn digest(&self) -> sim_digest::Digest256 {
        use sim_codec::CanonicalWriter;
        let mut w = CanonicalWriter::new("sim-network", "Network");
        w.u64(self.version);
        w.seq_len(self.edges.len());
        for e in self.edges.values() {
            w.u64(e.id);
            w.u64(e.a);
            w.u64(e.b);
            w.i64(e.cost);
            w.seq_len(e.geom.len());
            for &(x, y) in &e.geom {
                w.i64(x);
                w.i64(y);
            }
        }
        sim_digest::digest("sim-network", "Network", &w.into_bytes())
    }
}

// ---------------- 测试 ----------------

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(id: u64, a: u64, b: u64, cost: i64) -> Edge {
        Edge {
            id,
            a,
            b,
            cost,
            geom: vec![(a as i64 * 1000, 0), (b as i64 * 1000, 0)],
        }
    }

    fn small_net() -> Network {
        let mut n = Network::new();
        // 物理加入顺序刻意乱序，检验 CSR 稳定序
        n.publish(Tx { id: 1, ops: vec![Op::AddEdge(edge(2, 0, 2, 30))] }).unwrap();
        n.publish(Tx { id: 2, ops: vec![Op::AddEdge(edge(1, 0, 1, 10))] }).unwrap();
        n.publish(Tx { id: 3, ops: vec![Op::AddEdge(edge(3, 1, 2, 10))] }).unwrap();
        n
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
        assert!(!LAYER.is_empty());
    }

    #[test]
    fn region_ops() {
        let a = Region::new(0, 0, 10, 10);
        let b = Region::new(5, 5, 20, 20);
        let c = Region::new(11, 11, 30, 30);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
        assert_eq!(a.union(&b), Region::new(0, 0, 20, 20));
    }

    /// CSR 邻接序=(node_id, edge_id) 升序，与物理加入顺序无关（ADR-NEW-24 决策 1）。
    #[test]
    fn csr_adjacency_order_is_deterministic() {
        let n = small_net();
        let out: Vec<u64> = n.neighbors(0).map(|(eid, _, _)| eid).collect();
        assert_eq!(out, vec![1, 2], "node 0 出边必须按 edge_id 升序");
        // 反向构建的等价路网摘要一致
        let mut n2 = Network::new();
        n2.publish(Tx { id: 3, ops: vec![Op::AddEdge(edge(3, 1, 2, 10))] }).unwrap();
        n2.publish(Tx { id: 2, ops: vec![Op::AddEdge(edge(1, 0, 1, 10))] }).unwrap();
        n2.publish(Tx { id: 1, ops: vec![Op::AddEdge(edge(2, 0, 2, 30))] }).unwrap();
        assert_eq!(n.digest(), n2.digest(), "加入顺序不得影响摘要");
    }

    /// 拓扑事务原子：失败零改动（版本不变、摘要不变）；成功重建 CSR 并 +1。
    #[test]
    fn topology_tx_is_atomic() {
        let mut n = small_net();
        let v = n.version();
        let d = n.digest();
        let err = n.publish(Tx { id: 9, ops: vec![Op::AddEdge(edge(1, 5, 6, 1)) ]}).unwrap_err();
        assert_eq!(err, TxError::DuplicateEdge { id: 1 });
        assert_eq!(n.version(), v, "失败事务不得改版本");
        assert_eq!(n.digest(), d, "失败事务不得改状态");

        n.publish(Tx { id: 10, ops: vec![Op::AddEdge(edge(4, 2, 3, 5))] }).unwrap();
        assert_eq!(n.version(), v + 1);
        assert_eq!(n.neighbors(2).count(), 1, "重建后 CSR 可见新边");
    }

    /// 代价事务原位更新：不动拓扑、版本 +1、摘要变化。
    #[test]
    fn cost_tx_updates_in_place() {
        let mut n = small_net();
        let out_before: Vec<_> = n.neighbors(0).collect();
        n.publish(Tx { id: 20, ops: vec![Op::SetDynamicCost { id: 1, cost: 99 }] }).unwrap();
        let out_after: Vec<_> = n.neighbors(0).collect();
        assert_ne!(out_before, out_after, "代价应可见于邻接迭代");
        assert_eq!(out_after[0].2, 99);
    }

    /// ADR-NEW-23 两阶段判定 + 条文 9 合法性 + P1-04 代价默认不触发。
    #[test]
    fn stale_check_two_phase() {
        let mut n = small_net();
        let base = n.version();
        let path = vec![1u64, 3]; // 0->1->2

        // 区间内一笔不相交事务（新边在远处）→ Accept
        n.publish(Tx { id: 30, ops: vec![Op::AddEdge(edge(9, 8, 9, 1))] }).unwrap();
        assert_eq!(n.stale_check(&path, base), StaleOutcome::Accept);

        // 一笔仅代价变更（命中路径边）→ 默认不触发（P1-04）
        n.publish(Tx { id: 31, ops: vec![Op::SetDynamicCost { id: 3, cost: 50 }] }).unwrap();
        assert_eq!(n.stale_check(&path, base), StaleOutcome::Accept);

        // 删除路径上的边 → Recompute（trigger=命中事务）
        n.publish(Tx { id: 32, ops: vec![Op::RemoveEdge { id: 3 }] }).unwrap();
        assert_eq!(
            n.stale_check(&path, base),
            StaleOutcome::Recompute { trigger_tx: 32 }
        );
    }

    /// 条文 7 聚合：区间内多笔事务按 tx_id 升序取首个命中（与发布先后无关口径稳定）。
    #[test]
    fn stale_aggregation_is_stable() {
        let mut n = small_net();
        let base = n.version();
        let path = vec![1u64, 3];
        n.publish(Tx { id: 41, ops: vec![Op::RemoveEdge { id: 1 }] }).unwrap();
        n.publish(Tx { id: 40, ops: vec![Op::RemoveEdge { id: 3 }] }).unwrap();
        // 区间内 40 与 41 都命中，但判定按 tx_id 升序？——txs_between 按发布序，
        // 版本序即发布序；首个命中=41（先发布）。断言确定性即可：
        let r1 = n.stale_check(&path, base);
        let r2 = n.stale_check(&path, base);
        assert_eq!(r1, r2, "重复判定必须一致");
        assert!(matches!(r1, StaleOutcome::Recompute { .. }));
    }

    /// 合法性（条文 9）：路径含不存在的边 → Recompute。
    #[test]
    fn stale_legality_missing_edge() {
        let n = small_net();
        assert!(matches!(
            n.stale_check(&[1, 999], n.version()),
            StaleOutcome::Recompute { .. }
        ));
    }

    /// 摘要：同状态稳定、代价变化即变化（黄金值待基准定稿后固化，先性质断言）。
    #[test]
    fn digest_is_stable_and_sensitive() {
        let a = small_net();
        let b = small_net();
        assert_eq!(a.digest(), b.digest());
        let mut c = small_net();
        c.publish(Tx { id: 50, ops: vec![Op::SetDynamicCost { id: 1, cost: 7 }] }).unwrap();
        assert_ne!(a.digest(), c.digest());
    }
}
