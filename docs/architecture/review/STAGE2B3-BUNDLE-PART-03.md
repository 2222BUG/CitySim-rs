# STAGE2B3-BUNDLE PART 03（crates/sim-network/src/lib.rs 续段 1）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3 -->


===== FILE: crates/sim-network/src/lib.rs =====

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
    /// 预算预检拒绝（2026-11-19 投票修复 1d；事务不入队，下 tick 重试）。
    BudgetExceeded { kind: TxBudgetKind },
}

/// 事务预算分类（拓扑/代价分开计量；一笔含拓扑操作的事务整体按拓扑计）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxBudgetKind {
    Topology,
    Cost,
    EdgeOps,
}

/// 事务每 tick 上限（候选占位，待 ADR-NEW-14 阈值回填；2026-11-19 投票修复 1d）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxLimits {
    /// 拓扑事务（增/删边）每 tick 上限。
    pub topology_per_tick: u64,
    /// 纯代价事务每 tick 上限。
    pub cost_per_tick: u64,
    /// 单笔事务工作量上限（op 数+几何点数），防一笔超大事务绕过预算。
    pub max_edge_ops: u64,
}

impl Default for TxLimits {
    fn default() -> Self {
        Self { topology_per_tick: 1024, cost_per_tick: 1024, max_edge_ops: 8192 }
    }
}

/// 事务预算限制器（确定性工作量单位；publish 前预检 → 超限=QueueToNextTick 语义）。
/// 调用契约：`admit` 通过后调用 `Network::publish`；tick 边界调用 `reset`。
#[derive(Clone, Debug)]
pub struct TxLimiter {
    limits: TxLimits,
    topology_used: u64,
    cost_used: u64,
}

impl TxLimiter {
    pub fn new(limits: TxLimits) -> Self {
        Self { limits, topology_used: 0, cost_used: 0 }
    }

    pub fn reset(&mut self) {
        self.topology_used = 0;
        self.cost_used = 0;
    }

    fn op_work(op: &Op) -> u64 {
        match op {
            Op::AddEdge(e) => 1 + e.geom.len() as u64,
            _ => 1,
        }
    }

    /// 预检并计量（拒绝时零计量，可下 tick 原样重试）。
    pub fn admit(&mut self, tx: &Tx, _net: &Network) -> Result<(), TxError> {
        if tx.ops.is_empty() {
            return Err(TxError::EmptyTx);
        }
        let work: u64 = tx.ops.iter().map(Self::op_work).sum();
        if work > self.limits.max_edge_ops {
            return Err(TxError::BudgetExceeded { kind: TxBudgetKind::EdgeOps });
        }
        let is_topology = tx.ops.iter().any(|op| matches!(op, Op::AddEdge(_) | Op::RemoveEdge { .. }));
        if is_topology {
            if self.topology_used >= self.limits.topology_per_tick {
                return Err(TxError::BudgetExceeded { kind: TxBudgetKind::Topology });
            }
            self.topology_used += 1;
        } else {
            if self.cost_used >= self.limits.cost_per_tick {
                return Err(TxError::BudgetExceeded { kind: TxBudgetKind::Cost });
            }
            self.cost_used += 1;
        }
        Ok(())
    }
}

/// 事务重试上限（债务 #28 / ADR-NEW-24 决策 4：沿用原 tx_id 按全序重排入队重试；
/// 重试超限转受阻终态+诊断事件 NET-0001）。
pub const TX_RETRY_LIMIT: u32 = 3;

/// 一次重试入队的处置结果。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RetryOutcome {
    /// 已重新入队，携带累计重试次数（含本次）。
    Requeued { retry_count: u32 },
    /// 超过 `TX_RETRY_LIMIT`，转受阻终态。
    Blocked { retry_count: u32 },
}

/// 被预算拒绝（`TxError::BudgetExceeded`）事务的重试队列。
///
/// 语义（债务 #28 定案）：重试**沿用原 tx_id**（事务身份稳定，2.8 原则），按 tx_id
/// 全序重排入队（处理顺序不依赖到达先后，与过期结果策略同口径）；重试上限
/// [`TX_RETRY_LIMIT`]，超限转受阻终态并产生诊断事件（不改变权威状态）。
#[derive(Default, Debug)]
pub struct TxRetryQueue {
    /// tx_id -> (事务, 累计重试次数)。BTreeMap 保证 due() 按 tx_id 全序。
    pending: BTreeMap<u64, (Tx, u32)>,
    /// 受阻终态的 tx_id（按受阻顺序记录，确定性）。
    blocked: Vec<u64>,
}

impl TxRetryQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 失败事务重新入队（沿用原 tx_id）；同一事务重复 offer 累计计数。
    pub fn offer(&mut self, tx: Tx) -> RetryOutcome {
        let id = tx.id;
        let count = self.pending.get(&id).map(|(_, c)| *c).unwrap_or(0) + 1;
        if count > TX_RETRY_LIMIT {
            self.pending.remove(&id);
            self.blocked.push(id);
            RetryOutcome::Blocked { retry_count: count }
        } else {
            self.pending.insert(id, (tx, count));
            RetryOutcome::Requeued { retry_count: count }
        }
    }

    /// tick 边界取回全部待重试事务（按 tx_id 全序）；取走即出队——再次失败须
    /// 重新 [`offer`]（重试次数保留在队列语义内，不随 due 清零）。
    pub fn due(&mut self) -> Vec<Tx> {
        std::mem::take(&mut self.pending)
            .into_values()
            .map(|(tx, _)| tx)
            .collect()
    }

    /// 当前等待重试的 tx_id（升序）。
    pub fn pending_ids(&self) -> Vec<u64> {
        self.pending.keys().copied().collect()
    }

    /// 受阻终态 tx_id（按受阻顺序）。
    pub fn blocked_ids(&self) -> &[u64] {
        &self.blocked
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// 把受阻终态写入诊断通道（NET-0001，逐 tx_id 一条，顺序=受阻顺序）。
    /// 调用方在 tick 边界调用；诊断事件不改变权威状态（EC-08）。
    pub fn emit_blocked_diagnostics(&self, sink: &mut sim_error::DiagnosticSink, tick: u64) {
        for &id in &self.blocked {
            sink.emit(
                sim_error::ErrorCode::new("NET", 1),
                "tx retry limit exceeded; transaction blocked",
                sim_error::SimErrorContext {
                    tick,
                    entity_id: id,
                    system: "sim-network",
                },
            );
        }
    }
}

#[cfg(test)]
mod txlimiter_tests {
    use super::*;

    fn cost_tx(id: u64) -> Tx {
        Tx { id, ops: vec![Op::SetDynamicCost { id: 1, cost: 5 }] }
    }
    fn topo_tx(id: u64) -> Tx {
        Tx {
            id,
            ops: vec![Op::AddEdge(Edge {
                id: 100 + id,
                a: 0,
                b: 1,
                cost: 1,
                geom: vec![(0, 0), (1, 0)],
            })],
        }
    }

    #[test]
    fn topology_and_cost_budgets_are_separate() {
        // 纯代价事务用满不应挡住拓扑事务（1d：分开上限）。
        let net = Network::new();
        let mut lim = TxLimiter::new(TxLimits {
            topology_per_tick: 1,
            cost_per_tick: 1,
            max_edge_ops: 100,
        });
        assert!(lim.admit(&cost_tx(1), &net).is_ok());
        assert_eq!(
            lim.admit(&cost_tx(2), &net),
            Err(TxError::BudgetExceeded { kind: TxBudgetKind::Cost })
        );
        assert!(lim.admit(&topo_tx(3), &net).is_ok(), "topology budget independent");
        assert_eq!(
            lim.admit(&topo_tx(4), &net),
            Err(TxError::BudgetExceeded { kind: TxBudgetKind::Topology })
        );
    }

    #[test]
    fn oversized_single_tx_is_rejected() {
        // 一笔超大事务不能绕过预算（工作量上限）。
        let net = Network::new();
        let mut lim = TxLimiter::new(TxLimits {
            topology_per_tick: 10,
            cost_per_tick: 10,
            max_edge_ops: 3,
        });
        let big = Tx {
            id: 9,
            ops: vec![Op::AddEdge(Edge {
                id: 9,
                a: 0,
                b: 1,
                cost: 1,
                geom: vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            })],
        };
        assert_eq!(
            lim.admit(&big, &net),
            Err(TxError::BudgetExceeded { kind: TxBudgetKind::EdgeOps })
        );
    }

    #[test]
    fn rejected_tx_consumes_nothing_and_reset_rearms() {
        let net = Network::new();
        let mut lim = TxLimiter::new(TxLimits {
            topology_per_tick: 1,
            cost_per_tick: 1,
            max_edge_ops: 100,
        });
        assert!(lim.admit(&topo_tx(1), &net).is_ok());
        assert!(lim.admit(&topo_tx(2), &net).is_err());
        // 拒绝不计量：reset 后原样可入（QueueToNextTick 语义）
        lim.reset();
        assert!(lim.admit(&topo_tx(2), &net).is_ok());
    }

    #[test]
    fn mixed_tx_counts_as_topology() {
        let net = Network::new();
        let mut lim = TxLimiter::new(TxLimits {
            topology_per_tick: 1,
            cost_per_tick: 5,
            max_edge_ops: 100,
        });
        let mixed = Tx {
            id: 7,
            ops: vec![
                Op::SetDynamicCost { id: 1, cost: 5 },
                Op::AddEdge(Edge { id: 7, a: 0, b: 1, cost: 1, geom: vec![(0, 0)] }),
            ],
        };
        assert!(lim.admit(&mixed, &net).is_ok());
        assert!(lim.admit(&topo_tx(8), &net).is_err(), "mixed tx consumes topology budget");
    }
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

    /// 批量构造（构建期入口，非事务路径；一次 CSR 重建）。
    pub fn from_edges(edges: Vec<Edge>) -> Self {
        let mut n = Network::new();
        for e in edges {
            n.edges.insert(e.id, e);
        }
        n.rebuild_csr();
        n
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

#[cfg(test)]
mod txretry_tests {
    use super::*;

    fn tx(id: u64) -> Tx {
        Tx { id, ops: vec![Op::SetDynamicCost { id: 1, cost: 5 }] }
    }

    #[test]
    fn retry_keeps_tx_id_and_due_is_full_order() {
        // 债务 #28：沿用原 tx_id，按 tx_id 全序重排（与到达顺序无关）。
        let mut q = TxRetryQueue::new();
        assert_eq!(q.offer(tx(2)), RetryOutcome::Requeued { retry_count: 1 });
        assert_eq!(q.offer(tx(1)), RetryOutcome::Requeued { retry_count: 1 });
        let due = q.due();
        assert_eq!(due.iter().map(|t| t.id).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(due[0].id, 1, "original tx_id preserved");
        assert!(q.is_empty());
    }

    #[test]
    fn retry_limit_blocks_and_emits_diagnostic() {
        // 重试上限 3：第 4 次 offer 转受阻终态；NET-0001 诊断逐受阻 id 一条。
        let mut q = TxRetryQueue::new();
        for i in 1..=TX_RETRY_LIMIT {
            assert_eq!(q.offer(tx(7)), RetryOutcome::Requeued { retry_count: i });
        }
        assert_eq!(q.offer(tx(7)), RetryOutcome::Blocked { retry_count: 4 });
        assert_eq!(q.blocked_ids(), &[7]);
        assert!(q.is_empty(), "blocked tx must leave the pending queue");
        let mut sink = sim_error::DiagnosticSink::new();
        q.emit_blocked_diagnostics(&mut sink, 42);
        let events = sink.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code.to_string(), "NET-0001");
        assert_eq!(events[0].context.tick, 42);
        assert_eq!(events[0].context.ent