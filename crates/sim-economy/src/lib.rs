//! 经济及城市系统：承载后续城市模拟能力（主文档 2.2 架构分层）。
//! 权威crate。
//!
//! Stage 1 T1 骨架（2026-11-19 建立）——任务清单见 docs/architecture/Stage-1-Task-List.md。
//! 依赖方向约束见 docs/architecture/crate-authority-list.md 与 .github/scripts/dep_graph_check.py。

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "经济及城市系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

/// 经济数值上限（ADR-NEW-18/21 定案：经济类独立 i64，单位=最小货币单位 10^12）。
pub const ECONOMY_LIMIT: i64 = 1_000_000_000_000;

/// 经济事件（确定性序列，逐 tick 追加；入 digest）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EconomyEvent {
    TickSettled { tick: u64, income: i64, upkeep: i64, treasury: i64 },
    Paused { tick: u64 },
    Resumed { tick: u64 },
}

/// 经济错误（溢出/下溢防护：出错时状态不变，原子拒绝）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EconomyError {
    Overflow,
    Underflow,
    AlreadyPaused,
    NotPaused,
}

/// 经济账本：最小确定性循环（收入/维护结算+暂停语义+摘要）。
/// 收入含 event_u64 派生市场加成（ADR-NEW-22 "economy" 命名空间纯派生流，零状态）。
/// EC-04 L2/L3"暂停经济"语义：降级期由模拟引擎调用 pause/resume，暂停期不结算。
#[derive(Clone, Debug)]
pub struct EconomyLedger {
    seed: u64,
    tick: u64,
    treasury: i64,
    income_per_tick: i64,
    upkeep_per_tick: i64,
    market_range: u64,
    paused: bool,
    events: Vec<EconomyEvent>,
}

impl EconomyLedger {
    pub fn new(seed: u64, income_per_tick: i64, upkeep_per_tick: i64) -> Self {
        Self { seed, tick: 0, treasury: 0, income_per_tick, upkeep_per_tick, market_range: 1, paused: false, events: Vec::new() }
    }

    /// 市场加成区间（0..market_range，0=关闭派生加成）。
    pub fn with_market_range(mut self, market_range: u64) -> Self {
        self.market_range = market_range;
        self
    }

    pub fn treasury(&self) -> i64 { self.treasury }
    pub fn tick(&self) -> u64 { self.tick }
    pub fn paused(&self) -> bool { self.paused }
    pub fn events(&self) -> &[EconomyEvent] { &self.events }

    /// 结算当前 tick（paused 时跳过返回 None）。溢出/下溢原子拒绝。
    pub fn settle_tick(&mut self) -> Result<Option<EconomyEvent>, EconomyError> {
        if self.paused {
            return Ok(None);
        }
        let market = if self.market_range > 1 {
            (sim_rng::event_u64(self.seed, "economy", self.tick, 0) % self.market_range) as i64
        } else {
            0
        };
        let income = self.income_per_tick + market;
        let next = self.treasury.checked_add(income).ok_or(EconomyError::Overflow)?;
        let next = next.checked_sub(self.upkeep_per_tick).ok_or(EconomyError::Underflow)?;
        if !(0..=ECONOMY_LIMIT).contains(&next) {
            return Err(if next < 0 { EconomyError::Underflow } else { EconomyError::Overflow });
        }
        self.treasury = next;
        self.tick += 1;
        let ev = EconomyEvent::TickSettled { tick: self.tick, income, upkeep: self.upkeep_per_tick, treasury: self.treasury };
        self.events.push(ev.clone());
        Ok(Some(ev))
    }

    /// EC-04 L2/L3：暂停经济（幂等检查显式化，暂停期不结算不进 tick）。
    pub fn pause(&mut self, tick: u64) -> Result<(), EconomyError> {
        if self.paused {
            return Err(EconomyError::AlreadyPaused);
        }
        self.paused = true;
        self.events.push(EconomyEvent::Paused { tick });
        Ok(())
    }

    pub fn resume(&mut self, tick: u64) -> Result<(), EconomyError> {
        if !self.paused {
            return Err(EconomyError::NotPaused);
        }
        self.paused = false;
        self.events.push(EconomyEvent::Resumed { tick });
        Ok(())
    }

    /// 权威摘要（ADR-NEW-19 模块清单含经济；canonical+BLAKE3）。
    pub fn digest(&self) -> sim_digest::Digest256 {
        let mut w = sim_codec::CanonicalWriter::new("sim-economy", "EconomyLedger");
        w.u64(self.seed);
        w.u64(self.tick);
        w.i64(self.treasury);
        w.i64(self.income_per_tick);
        w.i64(self.upkeep_per_tick);
        w.u64(self.market_range);
        w.bool(self.paused);
        w.seq_len(self.events.len());
        for ev in &self.events {
            match ev {
                EconomyEvent::TickSettled { tick, income, upkeep, treasury } => {
                    w.u8(0); w.u64(*tick); w.i64(*income); w.i64(*upkeep); w.i64(*treasury);
                }
                EconomyEvent::Paused { tick } => { w.u8(1); w.u64(*tick); }
                EconomyEvent::Resumed { tick } => { w.u8(2); w.u64(*tick); }
            }
        }
        sim_digest::digest("sim-economy", "EconomyLedger", w.as_bytes())
    }
}

/// 引擎集成状态导出/恢复（integration-design.md §2）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EconomyState {
    pub seed: u64,
    pub tick: u64,
    pub treasury: i64,
    pub income_per_tick: i64,
    pub upkeep_per_tick: i64,
    pub market_range: u64,
    pub paused: bool,
    pub events: Vec<EconomyEvent>,
}

impl EconomyLedger {
    pub fn export_state(&self) -> EconomyState {
        EconomyState {
            seed: self.seed,
            tick: self.tick,
            treasury: self.treasury,
            income_per_tick: self.income_per_tick,
            upkeep_per_tick: self.upkeep_per_tick,
            market_range: self.market_range,
            paused: self.paused,
            events: self.events.clone(),
        }
    }

    pub fn restore_state(&mut self, s: EconomyState) {
        self.seed = s.seed;
        self.tick = s.tick;
        self.treasury = s.treasury;
        self.income_per_tick = s.income_per_tick;
        self.upkeep_per_tick = s.upkeep_per_tick;
        self.market_range = s.market_range;
        self.paused = s.paused;
        self.events = s.events;
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

    fn run(seed: u64) -> (EconomyLedger, Vec<EconomyEvent>) {
        let mut l = EconomyLedger::new(seed, 100, 30).with_market_range(20);
        let mut evs = Vec::new();
        for _ in 0..100 {
            if let Some(ev) = l.settle_tick().unwrap() {
                evs.push(ev);
            }
        }
        (l, evs)
    }

    #[test]
    fn two_runs_digest_equal() {
        let (l1, e1) = run(0x00C0_FFEE_1234_5678);
        let (l2, e2) = run(0x00C0_FFEE_1234_5678);
        assert_eq!(e1, e2);
        assert_eq!(l1.digest(), l2.digest());
        // 派生流确实产生收入变化（非退化）
        let varied = e1.iter().any(|ev| matches!(ev, EconomyEvent::TickSettled { income, .. } if *income != 100));
        assert!(varied);
    }

    #[test]
    fn treasury_bounds_are_enforced() {
        // 溢出：贴顶后下一 tick 仍入账 → Overflow 且状态不变
        let mut over = EconomyLedger::new(1, ECONOMY_LIMIT - 10, 0).with_market_range(1);
        assert!(over.settle_tick().unwrap().is_some());
        assert_eq!(over.treasury(), ECONOMY_LIMIT - 10);
        assert_eq!(over.settle_tick().unwrap_err(), EconomyError::Overflow);
        assert_eq!(over.treasury(), ECONOMY_LIMIT - 10); // 状态不变
        // 下溢：维护超国库 → Underflow 且状态不变
        let mut under = EconomyLedger::new(1, 0, 5);
        assert_eq!(under.settle_tick().unwrap_err(), EconomyError::Underflow);
        assert_eq!(under.treasury(), 0);
    }

    #[test]
    fn pause_resume_semantics() {
        let mut l = EconomyLedger::new(7, 100, 30).with_market_range(1);
        l.settle_tick().unwrap();
        let t = l.treasury();
        l.pause(1).unwrap();
        assert_eq!(l.pause(1).unwrap_err(), EconomyError::AlreadyPaused);
        for _ in 0..3 {
            assert_eq!(l.settle_tick().unwrap(), None); // 暂停期不结算
        }
        assert_eq!(l.treasury(), t);
        assert_eq!(l.tick(), 1);
        l.resume(4).unwrap();
        assert_eq!(l.resume(4).unwrap_err(), EconomyError::NotPaused);
        assert!(l.settle_tick().unwrap().is_some());
        assert_eq!(l.tick(), 2);
        // 事件序列含 Paused/Resumed 且入 digest（同事件不同 tick 位置 digest 不同）
        assert!(l.events().iter().any(|e| matches!(e, EconomyEvent::Paused { .. })));
        assert!(l.events().iter().any(|e| matches!(e, EconomyEvent::Resumed { .. })));
    }
}
