//! 预算注册表与确定性降级（ADR-NEW-14 C2 细化，Stage 2C C3a）。
//!
//! - 契约即代码：每系统在注册表声明计量单位/每 tick 上限/超限行为。
//! - 降级触发只依赖确定性工作量单位（预算消耗比例），禁止墙钟/负载/帧率。
//! - 降级等级与滚动预算是权威状态：入存档、入 GlobalDigest（G3 口径）。

use sim_codec::CanonicalWriter;
use sim_error::{ErrorCode, ErrorClass, SimError, SimErrorContext};

/// 计量单位（确定性工作量单位，ADR-NEW-14）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkUnit {
    /// A* 节点展开数（sim-pathfinding）。
    NodeExpansions,
    /// 事务应用数（sim-network）。
    TxApplies,
    /// 本 tick 更新实体数（sim-traffic 等）。
    EntityUpdates,
    /// 摘要/存档条目数（sim-save）。
    DigestEntries,
}

/// 超限行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowBehavior {
    /// 让步到下 tick 续算（寻路）。
    YieldToNextTick,
    /// 排队到下 tick（事务）。
    QueueToNextTick,
    /// 分片让步（实体更新）。
    ShardYield,
    /// Fatal：存档不可信（SAV-0003）。
    FatalSaveCorrupt,
}

/// 单个系统的预算声明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetDecl {
    pub system_id: u32,
    pub unit: WorkUnit,
    pub per_tick_cap: u64,
    pub overflow: OverflowBehavior,
}

/// 降级等级（EC-04 L1/L2/L3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DegradationLevel {
    Normal,
    L1,
    L2,
    L3,
}

/// 降级阈值（候选占位：70/90/100 万分比 + 恢复 500‰；ADR-NEW-14 C2 待负责人确认）。
#[derive(Debug, Clone, Copy)]
pub struct DegradationThresholds {
    /// 预算消耗比例 ≥ 此值触发（万分比：7000=70%）。
    pub l1_bp: u32,
    pub l2_bp: u32,
    pub l3_bp: u32,
    /// 恢复迟滞（万分比：5000=50%）。
    pub recover_bp: u32,
}

impl Default for DegradationThresholds {
    fn default() -> Self {
        Self { l1_bp: 7000, l2_bp: 9000, l3_bp: 10000, recover_bp: 5000 }
    }
}

/// 预算注册表 + 降级状态机。全部状态确定性、可入档。
#[derive(Debug, Clone)]
pub struct BudgetRegistry {
    decls: Vec<BudgetDecl>,
    /// 本 tick 各系统实耗（索引与 decls 对齐）。
    spent: Vec<u64>,
    /// 滚动预算状态：本 tick 全系统消耗比例（万分比，确定性派生）。
    rolling_used_bp: u32,
    pub level: DegradationLevel,
    thresholds: DegradationThresholds,
}

impl BudgetRegistry {
    pub fn new(decls: Vec<BudgetDecl>, thresholds: DegradationThresholds) -> Self {
        let spent = vec![0; decls.len()];
        Self { decls, spent, rolling_used_bp: 0, level: DegradationLevel::Normal, thresholds }
    }

    pub fn decls(&self) -> &[BudgetDecl] {
        &self.decls
    }

    pub fn level(&self) -> DegradationLevel {
        self.level
    }

    /// 记录实耗；返回是否超该系统每 tick 上限。
    pub fn spend(&mut self, system_id: u32, amount: u64) -> bool {
        let idx = self
            .decls
            .iter()
            .position(|d| d.system_id == system_id)
            .expect("undeclared system");
        self.spent[idx] = self.spent[idx].saturating_add(amount);
        self.spent[idx] > self.decls[idx].per_tick_cap
    }

    /// tick 边界：按确定性工作量单位更新降级等级，重置每 tick 计量。
    /// 返回新等级。触发/恢复只依赖 spent/cap 比例（工作量单位），无墙钟。
    pub fn end_tick(&mut self) -> DegradationLevel {
        let total_cap: u128 = self.decls.iter().map(|d| d.per_tick_cap as u128).sum();
        let total_spent: u128 = self.spent.iter().map(|s| *s as u128).sum();
        let used_bp: u32 = (total_spent.saturating_mul(10_000).checked_div(total_cap).unwrap_or(0))
            .min(10_000) as u32;
        self.rolling_used_bp = used_bp;
        let t = self.thresholds;
        self.level = match self.level {
            DegradationLevel::Normal => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else if used_bp >= t.l1_bp {
                    DegradationLevel::L1
                } else {
                    DegradationLevel::Normal
                }
            }
            DegradationLevel::L1 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else if used_bp < t.recover_bp {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L1
                }
            }
            DegradationLevel::L2 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp < t.recover_bp {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L2
                }
            }
            DegradationLevel::L3 => {
                if used_bp < t.recover_bp {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L3
                }
            }
        };
        self.spent.iter_mut().for_each(|s| *s = 0);
        self.level
    }

    /// 编码入档/入摘要（字段序即 canonical 序，G3 口径）。
    pub fn encode_state(&self, w: &mut CanonicalWriter) {
        w.u32(self.rolling_used_bp);
        w.u8(match self.level {
            DegradationLevel::Normal => 0,
            DegradationLevel::L1 => 1,
            DegradationLevel::L2 => 2,
            DegradationLevel::L3 => 3,
        });
    }

    /// 超限错误（供 FatalSaveCorrupt 类超限使用）。
    /// system_id 经 ctx.system/调用方日志定位（message 为 &'static str 无法插值）。
    pub fn overflow_error(&self, _system_id: u32, ctx: SimErrorContext) -> SimError {
        SimError {
            class: ErrorClass::Fatal,
            code: ErrorCode { module: "SAV", number: 3 },
            message: "digest entries exceeded per-tick cap; save untrustworthy",
            context: ctx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decls() -> Vec<BudgetDecl> {
        vec![
            BudgetDecl {
                system_id: 1,
                unit: WorkUnit::NodeExpansions,
                per_tick_cap: 100,
                overflow: OverflowBehavior::YieldToNextTick,
            },
            BudgetDecl {
                system_id: 2,
                unit: WorkUnit::TxApplies,
                per_tick_cap: 10,
                overflow: OverflowBehavior::QueueToNextTick,
            },
        ]
    }

    #[test]
    fn role_is_pinned() {
        assert!(crate::authoritative());
    }

    #[test]
    fn spend_and_overflow_flag() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        assert!(!r.spend(1, 100));
        assert!(r.spend(1, 1)); // 101 > 100
        assert!(!r.spend(2, 10));
        assert!(r.spend(2, 1));
    }

    #[test]
    fn degradation_levels_follow_workload_only() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        // 总 cap=110；消耗 77 → 7000bp → L1
        r.spend(1, 77);
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        // 重置后无消耗 → 恢复 Normal（< 5000bp）
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
        // 消耗 110 → 10000bp → L3
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 迟滞：消耗 60（5454bp）< recover 5000? 5454>=5000 → 保持 L3
        r.spend(1, 60);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 消耗 30（2727bp）< 5000 → 恢复
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
    }

    #[test]
    fn state_encoding_is_deterministic() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.spend(1, 90);
        r.end_tick();
        let mut w1 = CanonicalWriter::new("sim-core", "BudgetState");
        r.encode_state(&mut w1);
        let mut w2 = CanonicalWriter::new("sim-core", "BudgetState");
        r.encode_state(&mut w2);
        assert_eq!(w1.into_bytes(), w2.into_bytes());
    }

    #[test]
    fn unknown_system_spend_panics() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            r.spend(99, 1);
        }));
        assert!(result.is_err());
    }
}
