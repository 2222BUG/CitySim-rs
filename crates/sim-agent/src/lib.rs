//! Agent 系统：管理 Agent 身份、状态与行为（主文档 2.2 架构分层）。
//! 权威crate。
//!
//! Stage 1 T1 骨架（2026-11-19 建立）——任务清单见 docs/architecture/Stage-1-Task-List.md。
//! 依赖方向约束见 docs/architecture/crate-authority-list.md 与 .github/scripts/dep_graph_check.py。
//!
//! Stage 1 T4（2026-11-19）：Agent 身份——AgentId 与 Bevy Entity 解耦（2.3），
//! u64 单调分配器高水位入存档（EC-03/ADR-NEW-20：读档后不得复用 ID）。

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "Agent 系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

/// Agent 稳定身份（2.3）：与 Bevy Entity 解耦；u64 单调分配，入档=高水位。
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentId(u64);

impl AgentId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// AgentId 分配器：单调递增，高水位入存档（EC-03 身份类字段）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentIdAllocator {
    high_water: u64,
}

impl AgentIdAllocator {
    pub const fn new() -> Self {
        Self { high_water: 0 }
    }

    /// 分配下一个 AgentId（1 起单调递增，不复用）。
    pub fn alloc(&mut self) -> AgentId {
        self.high_water += 1;
        AgentId(self.high_water)
    }

    /// 已分配的最大 ID = 存档最小集"身份"高水位字段。
    pub const fn high_water(self) -> u64 {
        self.high_water
    }

    /// 读档恢复高水位；回退=ID 复用，拒绝。
    pub fn restore(&mut self, high_water: u64) {
        assert!(
            high_water >= self.high_water,
            "AgentId 高水位回退会导致身份复用"
        );
        self.high_water = high_water;
    }
}

/// Agent 生命周期状态（基础状态管理；状态机细化随 Stage 1 后续/债务 #7）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AgentStatus {
    Alive,
    Despawned,
}

/// Agent 登记表：BTreeMap 保证按 AgentId 稳定迭代序（2.9 条 4）。
#[derive(Clone, Debug, Default)]
pub struct AgentRegistry {
    agents: std::collections::BTreeMap<u64, AgentStatus>,
    alloc: AgentIdAllocator,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成新 Agent（身份由分配器给出）。
    pub fn spawn(&mut self) -> AgentId {
        let id = self.alloc.alloc();
        self.agents.insert(id.raw(), AgentStatus::Alive);
        id
    }

    /// 标记回收（保留身份记录，不删除 ID——防复用语义）。
    pub fn despawn(&mut self, id: AgentId) -> bool {
        match self.agents.get_mut(&id.raw()) {
            Some(status) => {
                *status = AgentStatus::Despawned;
                true
            }
            None => false,
        }
    }

    pub fn status(&self, id: AgentId) -> Option<AgentStatus> {
        self.agents.get(&id.raw()).copied()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// 按 AgentId 升序遍历（确定性迭代：BTreeMap 键序）。
    pub fn ids(&self) -> Vec<AgentId> {
        self.agents.keys().map(|k| AgentId(*k)).collect()
    }

    /// 身份高水位（入存档）。
    pub const fn id_high_water(&self) -> u64 {
        self.alloc.high_water
    }

    /// 读档恢复身份高水位（防复用）。
    pub fn restore_id_high_water(&mut self, high_water: u64) {
        self.alloc.restore(high_water);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
        assert!(!LAYER.is_empty());
    }

    /// AgentId 与 Entity 解耦：稳定 u64，单调分配。
    #[test]
    fn agent_ids_are_monotonic_u64() {
        let mut alloc = AgentIdAllocator::new();
        let a = alloc.alloc();
        let b = alloc.alloc();
        assert_eq!((a.raw(), b.raw()), (1, 2));
        assert_eq!(alloc.high_water(), 2);
    }

    /// 读档后高水位恢复，新身份不复用（EC-03/ADR-NEW-20）。
    #[test]
    fn restored_high_water_prevents_id_reuse() {
        let mut reg = AgentRegistry::new();
        let a1 = reg.spawn();
        let a2 = reg.spawn();
        assert_eq!((a1.raw(), a2.raw()), (1, 2));
        // 模拟读档：新表恢复高水位后 spawn 从 3 起。
        let mut restored = AgentRegistry::new();
        restored.restore_id_high_water(reg.id_high_water());
        let a3 = restored.spawn();
        assert_eq!(a3.raw(), 3);
    }

    /// 高水位回退必须拒绝。
    #[test]
    #[should_panic(expected = "AgentId 高水位回退会导致身份复用")]
    fn high_water_must_not_go_backwards() {
        let mut alloc = AgentIdAllocator::new();
        alloc.alloc();
        alloc.restore(0);
    }

    /// 登记表迭代确定性：按 AgentId 升序，与插入顺序无关。
    #[test]
    fn registry_iteration_is_stable_by_id() {
        let mut reg = AgentRegistry::new();
        let a = reg.spawn();
        let b = reg.spawn();
        let c = reg.spawn();
        assert_eq!(reg.ids(), vec![a, b, c]);
        // despawn 中间项不破坏键序。
        assert!(reg.despawn(b));
        assert_eq!(reg.ids(), vec![a, b, c]);
        assert_eq!(reg.status(b), Some(AgentStatus::Despawned));
        assert_eq!(reg.status(a), Some(AgentStatus::Alive));
    }
}

// ===================== Agent 状态机（Stage 3 T1，债务 #7） =====================
use std::collections::BTreeMap;
// RFC：docs/architecture/rfc-agent-state-machine.md（2026-11-22）。
// 与 AgentRegistry（粗粒度 Alive/Despawned）并存；集成期后合并，合并前以
// AgentLifecycle 为权威状态机。

/// 受阻原因（与触发源一一对应，RFC §2）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockedReason {
    /// 寻路不可达（FindOutcome::Unreachable；ADR-NEW-17 条文 10 兜底"标记受阻"）。
    Unreachable,
    /// 过期结果重算超限（recompute_limit，EC-01 策略块）。
    RecomputeLimitExceeded,
    /// 预算让步路径兜底 MarkBlocked（ADR-NEW-20/24 预算轴）。
    MarkBlockedFallback,
}

/// Agent 状态（RFC §2）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AgentState {
    /// 已分配 AgentId，等待进场。
    Spawning,
    /// 正常活动。
    Active,
    /// 受阻（前向进展不变量承诺的明确定义状态；attempts 单调递增保证可终止）。
    Blocked {
        reason: BlockedReason,
        since_tick: u64,
        attempts: u32,
    },
    /// 已离场（终态；ID 不复用）。
    Despawned,
}

/// 转移记录（确定性：按发生顺序追加；digest 覆盖）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StateTransition {
    pub agent_id: u64,
    pub from: AgentState,
    pub to: AgentState,
    pub tick: u64,
}

/// 非法转移（拒绝且不改变状态）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IllegalTransition {
    pub agent_id: u64,
    pub from: AgentState,
    pub to: AgentState,
}

/// `block()` 的处置结果。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockOutcome {
    /// 已受阻（含同状态再受阻 attempts+1）。
    Blocked { attempts: u32 },
    /// attempts 超过上限自动退役（RFC §4 前向进展收口）。
    Retired { attempts: u32 },
}

/// 默认受阻上限（与 recompute_limit、TX_RETRY_LIMIT 口径一致）。
pub const DEFAULT_BLOCK_LIMIT: u32 = 3;

/// Agent 状态机。转移全集见 RFC §3；未列出的转移一律 [`IllegalTransition`]。
#[derive(Debug)]
pub struct AgentLifecycle {
    states: BTreeMap<u64, AgentState>,
    log: Vec<StateTransition>,
    block_limit: u32,
}

impl Default for AgentLifecycle {
    fn default() -> Self {
        Self::new(DEFAULT_BLOCK_LIMIT)
    }
}

impl AgentLifecycle {
    pub fn new(block_limit: u32) -> Self {
        Self {
            states: BTreeMap::new(),
            log: Vec::new(),
            block_limit,
        }
    }

    pub fn state(&self, agent_id: u64) -> Option<AgentState> {
        self.states.get(&agent_id).copied()
    }

    pub fn transitions(&self) -> &[StateTransition] {
        &self.log
    }

    fn push(&mut self, agent_id: u64, from: AgentState, to: AgentState, tick: u64) {
        self.log.push(StateTransition {
            agent_id,
            from,
            to,
            tick,
        });
    }

    /// （无）→ Spawning。
    pub fn spawn(&mut self, agent_id: u64, tick: u64) -> Result<AgentState, IllegalTransition> {
        if let Some(&from) = self.states.get(&agent_id) {
            return Err(IllegalTransition {
                agent_id,
                from,
                to: AgentState::Spawning,
            });
        }
        self.states.insert(agent_id, AgentState::Spawning);
        self.push(agent_id, AgentState::Despawned, AgentState::Spawning, tick);
        Ok(AgentState::Spawning)
    }

    /// Spawning → Active。
    pub fn activate(&mut self, agent_id: u64, tick: u64) -> Result<AgentState, IllegalTransition> {
        self.transition_if(agent_id, tick, |s| matches!(s, AgentState::Spawning), AgentState::Active)
    }

    /// Active → Blocked；Blocked → Blocked（attempts+1）；attempts 超上限 → 自动
    /// 退役 Despawned（RFC §4）。
    pub fn block(
        &mut self,
        agent_id: u64,
        reason: BlockedReason,
        tick: u64,
    ) -> Result<BlockOutcome, IllegalTransition> {
        let from = self.ok_state(agent_id)?;
        let (attempts, base) = match from {
            AgentState::Active => (1u32, from),
            AgentState::Blocked {
                attempts,
                reason: _,
                since_tick: _,
            } => (attempts + 1, from),
            other => {
                return Err(IllegalTransition {
                    agent_id,
                    from: other,
                    to: AgentState::Blocked {
                        reason,
                        since_tick: tick,
                        attempts: 0,
                    },
                })
            }
        };
        if attempts > self.block_limit {
            let to = AgentState::Despawned;
            self.states.insert(agent_id, to);
            self.push(agent_id, base, to, tick);
            return Ok(BlockOutcome::Retired { attempts });
        }
        let to = AgentState::Blocked {
            reason,
            since_tick: tick,
            attempts,
        };
        self.states.insert(agent_id, to);
        self.push(agent_id, base, to, tick);
        Ok(BlockOutcome::Blocked { attempts })
    }

    /// Blocked → Active（重寻路成功）。
    pub fn unblock(&mut self, agent_id: u64, tick: u64) -> Result<AgentState, IllegalTransition> {
        self.transition_if(
            agent_id,
            tick,
            |s| matches!(s, AgentState::Blocked { .. }),
            AgentState::Active,
        )
    }

    /// 任意非终态 → Despawned（取消/正常离场/超时退役）。
    pub fn despawn(&mut self, agent_id: u64, tick: u64) -> Result<AgentState, IllegalTransition> {
        let from = self.ok_state(agent_id)?;
        if from == AgentState::Despawned {
            return Err(IllegalTransition {
                agent_id,
                from,
                to: AgentState::Despawned,
            });
        }
        self.states.insert(agent_id, AgentState::Despawned);
        self.push(agent_id, from, AgentState::Despawned, tick);
        Ok(AgentState::Despawned)
    }

    fn ok_state(&self, agent_id: u64) -> Result<AgentState, IllegalTransition> {
        self.states.get(&agent_id).copied().ok_or(IllegalTransition {
            agent_id,
            from: AgentState::Despawned,
            to: AgentState::Despawned,
        })
    }

    fn transition_if(
        &mut self,
        agent_id: u64,
        tick: u64,
        pred: impl FnOnce(&AgentState) -> bool,
        to: AgentState,
    ) -> Result<AgentState, IllegalTransition> {
        let from = self.ok_state(agent_id)?;
        if !pred(&from) {
            return Err(IllegalTransition {
                agent_id,
                from,
                to,
            });
        }
        self.states.insert(agent_id, to);
        self.push(agent_id, from, to, tick);
        Ok(to)
    }

    /// 受阻转移写入诊断通道（AGT-0001，逐条；不改变权威状态，EC-08）。
    pub fn emit_block_diagnostics(&self, sink: &mut sim_error::DiagnosticSink) {
        for t in &self.log {
            if matches!(t.to, AgentState::Blocked { .. }) {
                sink.emit(
                    sim_error::ErrorCode::new("AGT", 1),
                    "agent entered blocked state",
                    sim_error::SimErrorContext {
                        tick: t.tick,
                        entity_id: t.agent_id,
                        system: "sim-agent",
                    },
                );
            }
        }
    }

    /// 转移序列摘要（两跑 digest 相等=状态机级可复现；RFC §5）。
    pub fn digest(&self) -> sim_digest::Digest256 {
        let mut w = sim_codec::CanonicalWriter::new("sim-agent", "AgentLifecycle");
        w.seq_len(self.log.len());
        for t in &self.log {
            w.u64(t.agent_id);
            w.u8(match t.from {
                AgentState::Spawning => 0,
                AgentState::Active => 1,
                AgentState::Blocked { .. } => 2,
                AgentState::Despawned => 3,
            });
            w.u8(match t.to {
                AgentState::Spawning => 0,
                AgentState::Active => 1,
                AgentState::Blocked { .. } => 2,
                AgentState::Despawned => 3,
            });
            if let AgentState::Blocked {
                reason,
                since_tick,
                attempts,
            } = t.to
            {
                w.u8(match reason {
                    BlockedReason::Unreachable => 0,
                    BlockedReason::RecomputeLimitExceeded => 1,
                    BlockedReason::MarkBlockedFallback => 2,
                });
                w.u64(since_tick);
                w.u32(attempts);
            }
            w.u64(t.tick);
        }
        sim_digest::digest("sim-agent", "AgentLifecycle", w.as_bytes())
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    #[test]
    fn transition_table_coverage() {
        // RFC §3 全集合法转移逐一可达 + 典型非法转移被拒。
        let mut lc = AgentLifecycle::new(3);
        assert_eq!(lc.spawn(1, 0), Ok(AgentState::Spawning));
        assert_eq!(lc.activate(1, 1), Ok(AgentState::Active));
        assert_eq!(
            lc.block(1, BlockedReason::Unreachable, 2),
            Ok(BlockOutcome::Blocked { attempts: 1 })
        );
        assert_eq!(lc.unblock(1, 3), Ok(AgentState::Active));
        assert_eq!(
            lc.block(1, BlockedReason::RecomputeLimitExceeded, 4),
            Ok(BlockOutcome::Blocked { attempts: 1 })
        );
        assert_eq!(
            lc.block(1, BlockedReason::MarkBlockedFallback, 5),
            Ok(BlockOutcome::Blocked { attempts: 2 })
        );
        assert_eq!(lc.despawn(1, 6), Ok(AgentState::Despawned));
        // 非法：终态复活、Spawning 直接 block、Active unblock、未登记操作。
        assert!(lc.despawn(1, 7).is_err());
        assert!(lc.activate(1, 7).is_err());
        let mut lc2 = AgentLifecycle::new(3);
        assert!(lc2.unblock(9, 0).is_err(), "unregistered agent");
        lc2.spawn(2, 0).unwrap();
        assert!(lc2.block(2, BlockedReason::Unreachable, 1).is_err());
        lc2.activate(2, 2).unwrap();
        assert!(lc2.unblock(2, 3).is_err(), "Active cannot unblock");
    }

    #[test]
    fn blocked_enters_and_exits_with_limit() {
        // 受阻进入/退出 + 上限退役（前向进展收口，RFC §4）。
        let mut lc = AgentLifecycle::new(2);
        lc.spawn(1, 0).unwrap();
        lc.activate(1, 1).unwrap();
        assert_eq!(
            lc.block(1, BlockedReason::Unreachable, 2),
            Ok(BlockOutcome::Blocked { attempts: 1 })
        );
        assert_eq!(
            lc.block(1, BlockedReason::MarkBlockedFallback, 3),
            Ok(BlockOutcome::Blocked { attempts: 2 })
        );
        // 第 3 次（attempts=3 > limit=2）→ 自动退役。
        assert_eq!(
            lc.block(1, BlockedReason::Unreachable, 4),
            Ok(BlockOutcome::Retired { attempts: 3 })
        );
        assert_eq!(lc.state(1), Some(AgentState::Despawned));
        // 对照：unblock 路径正常回到 Active。
        let mut lc2 = AgentLifecycle::new(3);
        lc2.spawn(1, 0).unwrap();
        lc2.activate(1, 1).unwrap();
        lc2.block(1, BlockedReason::Unreachable, 2).unwrap();
        assert_eq!(lc2.unblock(1, 5), Ok(AgentState::Active));
    }

    #[test]
    fn transition_sequence_digest_is_reproducible() {
        // 两跑转移序列 digest 一致（RFC §5）。
        let run = || {
            let mut lc = AgentLifecycle::new(3);
            lc.spawn(1, 0).unwrap();
            lc.spawn(2, 0).unwrap();
            lc.activate(1, 1).unwrap();
            lc.activate(2, 1).unwrap();
            lc.block(1, BlockedReason::Unreachable, 2).unwrap();
            lc.unblock(1, 3).unwrap();
            lc.block(2, BlockedReason::RecomputeLimitExceeded, 4).unwrap();
            lc.despawn(2, 5).unwrap();
            lc.digest()
        };
        assert_eq!(run(), run());
        let mut sink = sim_error::DiagnosticSink::new();
        let mut lc = AgentLifecycle::new(3);
        lc.spawn(1, 0).unwrap();
        lc.activate(1, 1).unwrap();
        lc.block(1, BlockedReason::Unreachable, 2).unwrap();
        lc.emit_block_diagnostics(&mut sink);
        let events = sink.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code.to_string(), "AGT-0001");
    }
}
