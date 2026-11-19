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

/// 降级阈值（候选占位：70/90/100 万分比 + 恢复 5000bp；ADR-NEW-14 C2 待负责人确认）。
/// 2026-11-19 投票修复（Claude）：触发/恢复加连续 tick 要求，防单 tick 突发误触发/误恢复。
#[derive(Debug, Clone, Copy)]
pub struct DegradationThresholds {
    /// 预算消耗比例 ≥ 此值触发（万分比：7000=70%）。触发比较为闭区间（>=）。
    pub l1_bp: u32,
    pub l2_bp: u32,
    pub l3_bp: u32,
    /// 恢复迟滞（万分比：5000=50%）。恢复比较为开区间（<）。
    pub recover_bp: u32,
    /// 进入降级所需连续超阈 tick 数（候选占位=2；单 tick 突发不触发）。
    pub trigger_ticks: u32,
    /// 恢复 Normal 所需连续低于 recover_bp 的 tick 数（候选占位=2）。
    pub recover_ticks: u32,
}

impl Default for DegradationThresholds {
    fn default() -> Self {
        Self {
            l1_bp: 7000,
            l2_bp: 9000,
            l3_bp: 10000,
            recover_bp: 5000,
            trigger_ticks: 2,
            recover_ticks: 2,
        }
    }
}

/// 预算注册表 + 降级状态机。全部状态确定性、可入档。
#[derive(Debug, Clone)]
pub struct BudgetRegistry {
    decls: Vec<BudgetDecl>,
    /// 本 tick 各系统实耗（索引与 decls 对齐）。
    spent: Vec<u64>,
    /// 本 tick 各系统预算占比的最大值（万分比）。2026-11-19 投票修复（MiniMax/GPT-6）：
    /// 聚合口径=逐系统比例取最严（旧口径按 cap 加权总和，单系统饱和会被空闲系统稀释）。
    rolling_used_bp: u32,
    /// 连续超阈 tick 计数（触发门）。
    over_streak: u32,
    /// 连续低于恢复线 tick 计数（恢复门）。
    under_streak: u32,
    /// 本 tick 外部需求压力（万分比，0..=10000；MFD→降级信号通道，ADR-NEW-26 草案）。
    demand_bp: u32,
    /// 本 tick 需求压力的归档口径（X5 审计修复：demand_bp 在 end_tick 清零，
    /// 直接入档恒 0——last_demand_bp 跨 end_tick 保留供 encode_state 归档）。
    last_demand_bp: u32,
    /// 连续高需求 tick 计数（与 over_streak 同门限语义）。
    demand_streak: u32,
    pub level: DegradationLevel,
    thresholds: DegradationThresholds,
}

impl BudgetRegistry {
    pub fn new(decls: Vec<BudgetDecl>, thresholds: DegradationThresholds) -> Self {
        let spent = vec![0; decls.len()];
        Self {
            decls,
            spent,
            rolling_used_bp: 0,
            over_streak: 0,
            under_streak: 0,
            demand_bp: 0,
            last_demand_bp: 0,
            demand_streak: 0,
            level: DegradationLevel::Normal,
            thresholds,
        }
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

    /// 记录本 tick 外部需求压力（万分比；确定性定点，来自权威状态导出——
    /// 如 sim-traffic 密度/k_c 比值。ADR-NEW-26 草案通道；每 tick 末清零）。
    pub fn note_demand_pressure(&mut self, pressure_bp: u32) {
        self.demand_bp = pressure_bp.min(10_000);
        self.last_demand_bp = self.demand_bp;
    }

    /// tick 边界：按确定性工作量单位更新降级等级，重置每 tick 计量。
    /// 返回新等级。触发/恢复只依赖 spent/cap 比例（工作量单位），无墙钟。
    ///
    /// 2026-11-19 投票修复：
    /// - 聚合口径=逐系统比例取最严（max_i spent_i/cap_i），单系统饱和不被稀释；
    /// - 触发（Normal→降级）需连续 trigger_ticks 个超阈 tick（触发比较闭区间 >=）；
    /// - 恢复（→Normal）需连续 recover_ticks 个低于 recover_bp 的 tick（开区间 <）；
    /// - 升级（L1→L2→L3）保持即时（安全方向不加延迟）。
    pub fn end_tick(&mut self) -> DegradationLevel {
        let used_bp: u32 = self
            .decls
            .iter()
            .zip(self.spent.iter())
            .map(|(d, s)| {
                if d.per_tick_cap == 0 {
                    if *s > 0 { 10_000u32 } else { 0 }
                } else {
                    ((*s as u128).saturating_mul(10_000) / d.per_tick_cap as u128).min(10_000) as u32
                }
            })
            .max()
            .unwrap_or(0);
        self.rolling_used_bp = used_bp;
        let t = self.thresholds;
        // 触发/恢复连续计数（确定性派生状态）：有效压力 = max(工作量占比, 外部需求压力)。
        // 需求压力参与触发/恢复门，但单独触发时降级封顶 L1（模型形式未裁决=债务 #31，
        // 限制影响面；工作量压力仍可推 L2/L3）。
        let effective_bp = used_bp.max(self.demand_bp);
        if effective_bp >= t.l1_bp {
            self.over_streak = self.over_streak.saturating_add(1);
            self.demand_streak = if self.demand_bp >= t.l1_bp {
                self.demand_streak.saturating_add(1)
            } else {
                0
            };
        } else {
            self.over_streak = 0;
            self.demand_streak = 0;
        }
        if effective_bp < t.recover_bp {
            self.under_streak = self.under_streak.saturating_add(1);
        } else {
            self.under_streak = 0;
        }
        let triggered = self.over_streak >= t.trigger_ticks;
        let demand_only_trigger = triggered
            && used_bp < t.l1_bp
            && self.demand_streak >= t.trigger_ticks;
        let recovered = self.under_streak >= t.recover_ticks;
        self.level = match self.level {
            DegradationLevel::Normal => {
                if !triggered {
                    DegradationLevel::Normal
                } else if demand_only_trigger {
                    DegradationLevel::L1
                } else if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else {
                    DegradationLevel::L1
                }
            }
            DegradationLevel::L1 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L1
                }
            }
            DegradationLevel::L2 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L2
                }
            }
            DegradationLevel::L3 => {
                if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L3
                }
            }
        };
        self.spent.iter_mut().for_each(|s| *s = 0);
        self.demand_bp = 0;
        self.level
    }

    /// 编码入档/入摘要（字段序即 canonical 序，G3 口径）。
    pub fn encode_state(&self, w: &mut CanonicalWriter) {
        w.u32(self.rolling_used_bp);
        w.u32(self.over_streak);
        w.u32(self.under_streak);
        w.u32(self.last_demand_bp);
        w.u32(self.demand_streak);
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
        // trigger_ticks=1：纯比例路径（旧口径测试迁移）
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        // 逐系统取最严：系统 1 消耗 77/100 → 7700bp ≥7000 → L1
        r.spend(1, 77);
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        // 重置后无消耗 → 恢复 Normal（< 5000bp）
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
        // 系统 1 消耗 110 → 10000bp → L3
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 迟滞：60/100=6000bp ≥5000 → 保持 L3
        r.spend(1, 60);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 30/100=3000bp < 5000 → 恢复
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
    }

    #[test]
    fn single_system_saturation_is_not_diluted() {
        // 2026-11-19 投票修复（MiniMax/GPT-6）回归：旧 cap 加权口径下
        // 系统 2（cap=10）打满只贡献 10/110≈9%→永不触发；新口径取最严必须触发。
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        r.spend(2, 10); // 系统 2 满负荷 10000bp，系统 1 空闲
        assert_eq!(r.end_tick(), DegradationLevel::L3);
    }

    #[test]
    fn threshold_boundaries_are_closed_open() {
        // GPT-6 边界要求：恰 70/90/100 触发（闭区间 >=），恰 50 不恢复（开区间 <）。
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        r.spend(1, 70); // 7000bp 恰= l1
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        r.spend(1, 90); // 9000bp 恰= l2
        assert_eq!(r.end_tick(), DegradationLevel::L2);
        r.spend(1, 100); // 10000bp 恰= l3
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        r.spend(1, 50); // 5000bp 恰= recover → 不恢复（开区间）
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        r.spend(1, 49); // 4900bp < 5000 → 恢复
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
    }

    #[test]
    fn trigger_and_recover_need_consecutive_ticks() {
        // Claude：单 tick 突发不触发/不恢复（默认 trigger_ticks=recover_ticks=2）。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 1 个超阈 tick：不触发
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::L3); // 第 2 个：触发
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::L3); // 第 1 个低消耗 tick：不恢复
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 2 个：恢复
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

    #[test]
    fn demand_pressure_triggers_l1_only_with_hysteresis() {
        // MFD→降级通道（ADR-NEW-26 草案）：需求压力参与触发/恢复门（effective=max），
        // 单独触发封顶 L1；恢复需需求与工作量同时回落。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.note_demand_pressure(10_000);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 1 tick：触发门未到
        r.note_demand_pressure(10_000);
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        r.note_demand_pressure(0);
        assert_eq!(r.end_tick(), DegradationLevel::L1); // 第 1 个低需求 tick：不恢复
        r.note_demand_pressure(0);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 2 个：恢复
    }

    #[test]
    fn demand_alone_cannot_reach_l2_l3() {
        // 需求压力单独触发封顶 L1（债务 #31 未裁决前限制影响面）；
        // 工作量压力仍可推 L2/L3。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        for _ in 0..2 {
            r.note_demand_pressure(10_000);
            assert_ne!(r.end_tick(), DegradationLevel::L2);
        }
        assert_eq!(r.level(), DegradationLevel::L1);
        for _ in 0..2 {
            r.spend(1, 2000);
            r.end_tick();
        }
        assert_eq!(r.level(), DegradationLevel::L3);
    }

    #[test]
    fn demand_state_is_covered_by_encode() {
        // 需求通道状态入档/入摘要（G3）：demand_bp 敏感。
        let mk = |demand: u32| {
            let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
            r.note_demand_pressure(demand);
            let mut w = CanonicalWriter::new("sim-core", "BudgetState");
            r.encode_state(&mut w);
            w.into_bytes()
        };
        assert_ne!(mk(0), mk(10_000), "demand_bp must be covered");
    }

    #[test]
    fn demand_archival_survives_end_tick() {
        // X5 审计修复：encode_state 归档 last_demand_bp（end_tick 清零 demand_bp
        // 后仍可归档——旧口径入档恒 0）。
        let decls = vec![BudgetDecl { system_id: 1, unit: WorkUnit::NodeExpansions, per_tick_cap: 1000, overflow: OverflowBehavior::YieldToNextTick }];
        let mk = |demand: u32| {
            let mut r = BudgetRegistry::new(decls.clone(), DegradationThresholds::default());
            r.note_demand_pressure(demand);
            r.end_tick(); // 清零 demand_bp 后归档
            let mut w = CanonicalWriter::new("sim-core", "Budget");
            r.encode_state(&mut w);
            w.into_bytes()
        };
        assert_ne!(mk(0), mk(10_000), "last_demand_bp must survive end_tick");
    }
}
