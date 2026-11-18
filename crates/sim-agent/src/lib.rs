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
