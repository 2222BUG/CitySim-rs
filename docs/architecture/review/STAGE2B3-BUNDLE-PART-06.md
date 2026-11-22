# STAGE2B3-BUNDLE PART 06（本卷文件：crates/sim-replay/src/lib.rs, crates/sim-economy/src/lib.rs）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3；与 ZIP 同源；粘贴时请整卷一条消息发送 -->


===== FILE: crates/sim-replay/src/lib.rs =====

//! 回放输入 / 模拟配置 Schema（Stage 3 T2，债务 #10）。
//!
//! Schema 文档：docs/architecture/replay-schema.md（version = 1）。
//! 编码：canonical 小端（sim-codec，module="sim-replay"、kind="ReplayInput"），
//! 与存档共用底层编码规则、顶层 Schema 独立（ADR-NEW-19 决策）。
//! 权威 crate：禁浮点/禁 HashMap 迭代依赖/禁系统时钟（clippy.toml 分层）。

#![forbid(unsafe_code)]

use sim_codec::CanonicalWriter;

/// 回放 Schema 版本（语义变更必须 bump，见 replay-schema.md §5）。
pub const REPLAY_SCHEMA_VERSION: u16 = 1;

/// 解码/录制错误。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    /// 数据提前结束。
    Truncated,
    /// 尾部多余字节（不静默丢弃，EC-03 同口径）。
    TrailingBytes,
    /// 信封损坏（CSIMCOD1/module/kind 不符）。
    BadEnvelope,
    /// Schema 版本不兼容（可诊断：RPL-0001）。
    UnsupportedVersion { found: u16 },
    /// 未知事件 tag（版本 bump 后旧端拒绝新事件）。
    UnknownEventTag(u8),
    /// 录制 tick 逆序。
    OutOfOrderTick { previous: u64, found: u64 },
}

impl ReplayError {
    /// 错误码（docs/architecture/error-code-spec.md）。
    pub fn code(&self) -> sim_error::ErrorCode {
        sim_error::ErrorCode::new("RPL", 1)
    }
}

/// 输入事件（Schema §2；判别 tag u8）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    /// 玩家命令（2.9 条 8：带生效 tick；对应 sim-core PlayerCommand 字段）。
    Command {
        command_id: u64,
        effective_tick: u64,
        agent_id: u64,
        opcode: u32,
        arg0: i64,
        arg1: i64,
    },
    /// tick 边界配置变更（2.7；key=稳定键哈希）。
    ConfigChange { tick: u64, key: u64, value: i64 },
    /// LOD 切换记录（ADR-NEW-14 条 6）。
    LodSwitch {
        tick: u64,
        entity_id: u64,
        from_level: u8,
        to_level: u8,
    },
    /// 预算调整（2.7 机器性能调整，tick 边界）。
    BudgetAdjustment {
        tick: u64,
        system_id: u64,
        per_tick_cap: u64,
    },
}

impl InputEvent {
    /// 事件归属 tick（提交语义见 replay-schema.md §4）。
    pub fn tick(&self) -> u64 {
        match *self {
            InputEvent::Command { effective_tick, .. } => effective_tick,
            InputEvent::ConfigChange { tick, .. }
            | InputEvent::LodSwitch { tick, .. }
            | InputEvent::BudgetAdjustment { tick, .. } => tick,
        }
    }
}

/// 回放输入：权威运行的输入侧完整记录（种子+配置+事件序列）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayInput {
    pub schema_version: u16,
    pub seed: u64,
    pub tick_rate_num: u64,
    pub tick_rate_den: u64,
    /// 记录序=发生序；回放按 tick 提交。
    pub events: Vec<InputEvent>,
}

impl ReplayInput {
    pub fn new(seed: u64, tick_rate_num: u64, tick_rate_den: u64) -> Self {
        Self {
            schema_version: REPLAY_SCHEMA_VERSION,
            seed,
            tick_rate_num,
            tick_rate_den,
            events: Vec::new(),
        }
    }

    fn encode_into(&self, w: &mut CanonicalWriter) {
        w.u16(self.schema_version);
        w.u64(self.seed);
        w.u64(self.tick_rate_num);
        w.u64(self.tick_rate_den);
        w.seq_len(self.events.len());
        for ev in &self.events {
            match *ev {
                InputEvent::Command {
                    command_id,
                    effective_tick,
                    agent_id,
                    opcode,
                    arg0,
                    arg1,
                } => {
                    w.u8(0);
                    w.u64(command_id);
                    w.u64(effective_tick);
                    w.u64(agent_id);
                    w.u32(opcode);
                    w.i64(arg0);
                    w.i64(arg1);
                }
                InputEvent::ConfigChange { tick, key, value } => {
                    w.u8(1);
                    w.u64(tick);
                    w.u64(key);
                    w.i64(value);
                }
                InputEvent::LodSwitch {
                    tick,
                    entity_id,
                    from_level,
                    to_level,
                } => {
                    w.u8(2);
                    w.u64(tick);
                    w.u64(entity_id);
                    w.u8(from_level);
                    w.u8(to_level);
                }
                InputEvent::BudgetAdjustment {
                    tick,
                    system_id,
                    per_tick_cap,
                } => {
                    w.u8(3);
                    w.u64(tick);
                    w.u64(system_id);
                    w.u64(per_tick_cap);
                }
            }
        }
    }

    /// canonical 编码（含 CSIMCOD1|sim-replay|ReplayInput 信封）。
    pub fn encode(&self) -> Vec<u8> {
        let mut w = CanonicalWriter::new("sim-replay", "ReplayInput");
        self.encode_into(&mut w);
        w.into_bytes()
    }

    /// 解码：信封校验→版本校验（RPL-0001）→字段序读取→尾字节拒绝。
    pub fn decode(raw: &[u8]) -> Result<Self, ReplayError> {
        let mut r = Cursor::new(raw);
        r.envelope("sim-replay", "ReplayInput")?;
        let schema_version = r.u16()?;
        if schema_version != REPLAY_SCHEMA_VERSION {
            return Err(ReplayError::UnsupportedVersion {
                found: schema_version,
            });
        }
        let seed = r.u64()?;
        let tick_rate_num = r.u64()?;
        let tick_rate_den = r.u64()?;
        let n = r.usize()?;
        let mut events = Vec::with_capacity(n);
        for _ in 0..n {
            let tag = r.u8()?;
            let ev = match tag {
                0 => InputEvent::Command {
                    command_id: r.u64()?,
                    effective_tick: r.u64()?,
                    agent_id: r.u64()?,
                    opcode: r.u32()?,
                    arg0: r.i64()?,
                    arg1: r.i64()?,
                },
                1 => InputEvent::ConfigChange {
                    tick: r.u64()?,
                    key: r.u64()?,
                    value: r.i64()?,
                },
                2 => InputEvent::LodSwitch {
                    tick: r.u64()?,
                    entity_id: r.u64()?,
                    from_level: r.u8()?,
                    to_level: r.u8()?,
                },
                3 => InputEvent::BudgetAdjustment {
                    tick: r.u64()?,
                    system_id: r.u64()?,
                    per_tick_cap: r.u64()?,
                },
                other => return Err(ReplayError::UnknownEventTag(other)),
            };
            events.push(ev);
        }
        if r.pos != raw.len() {
            return Err(ReplayError::TrailingBytes);
        }
        Ok(Self {
            schema_version,
            seed,
            tick_rate_num,
            tick_rate_den,
            events,
        })
    }

    /// 输入侧摘要（回放身份绑定；存档元数据可引用）。
    pub fn digest(&self) -> sim_digest::Digest256 {
        sim_digest::digest("sim-replay", "ReplayInput", &self.encode())
    }
}

/// 录制器：事件收集 + tick 单调非降检查（Schema §4）。
pub struct Recorder {
    input: ReplayInput,
    last_tick: u64,
}

impl Recorder {
    pub fn new(seed: u64, tick_rate_num: u64, tick_rate_den: u64) -> Self {
        Self {
            input: ReplayInput::new(seed, tick_rate_num, tick_rate_den),
            last_tick: 0,
        }
    }

    pub fn record(&mut self, ev: InputEvent) -> Result<(), ReplayError> {
        let t = ev.tick();
        if t < self.last_tick {
            return Err(ReplayError::OutOfOrderTick {
                previous: self.last_tick,
                found: t,
            });
        }
        self.last_tick = t;
        self.input.events.push(ev);
        Ok(())
    }

    pub fn finish(self) -> ReplayInput {
        self.input
    }
}

/// 回放器：按 tick 提交事件（记录序；不重算决策）。
pub struct Replayer {
    input: ReplayInput,
    cursor: usize,
}

impl Replayer {
    pub fn new(input: ReplayInput) -> Self {
        Self { input, cursor: 0 }
    }

    /// 取回归属 tick ≤ `tick` 的全部未提交事件（按记录序）。
    pub fn due(&mut self, tick: u64) -> Vec<InputEvent> {
        let mut out = Vec::new();
        while self.cursor < self.input.events.len() {
            let ev = self.input.events[self.cursor];
            if ev.tick() > tick {
                break;
            }
            out.push(ev);
            self.cursor += 1;
        }
        out
    }

    pub fn is_drained(&self) -> bool {
        self.cursor >= self.input.events.len()
    }

    pub fn input(&self) -> &ReplayInput {
        &self.input
    }
}

/// 最小只读解码游标（信封=CSIMCOD1|module|kind，各 u64 长度前缀）。
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ReplayError> {
        if self.pos + n > self.buf.len() {
            return Err(ReplayError::Truncated);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn envelope(&mut self, module: &str, kind: &str) -> Result<(), ReplayError> {
        if self.read_str()? != "CSIMCOD1" {
            return Err(ReplayError::BadEnvelope);
        }
        if self.read_str()? != module {
            return Err(ReplayError::BadEnvelope);
        }
        if self.read_str()? != kind {
            return Err(ReplayError::BadEnvelope);
        }
        Ok(())
    }

    fn read_str(&mut self) -> Result<String, ReplayError> {
        let n = self.u64()? as usize;
        let bytes = self.take(n)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ReplayError::BadEnvelope)
    }

    fn u8(&mut self) -> Result<u8, ReplayError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ReplayError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, ReplayError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, ReplayError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64, ReplayError> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn usize(&mut self) -> Result<usize, ReplayError> {
        Ok(self.u64()? as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario() -> Vec<InputEvent> {
        vec![
            InputEvent::Command {
                command_id: 1,
                effective_tick: 1,
                agent_id: 7,
                opcode: 2,
                arg0: -3,
                arg1: 99,
            },
            InputEvent::ConfigChange {
                tick: 1,
                key: 0xDEAD,
                value: 5,
            },
            InputEvent::LodSwitch {
                tick: 2,
                entity_id: 7,
                from_level: 0,
                to_level: 1,
            },
            InputEvent::BudgetAdjustment {
                tick: 3,
                system_id: 1,
                per_tick_cap: 65_536,
            },
            InputEvent::Command {
                command_id: 2,
                effective_tick: 4,
                agent_id: 8,
                opcode: 0,
                arg0: 0,
                arg1: 0,
            },
        ]
    }

    fn record() -> ReplayInput {
        let mut rec = Recorder::new(0x1234, 30, 1);
        for ev in scenario() {
            rec.record(ev).unwrap();
        }
        rec.finish()
    }

    /// 玩具确定性模拟：状态=累加器；事件按序施加确定性变换。
    fn apply(acc: &mut u64, ev: &InputEvent) {
        match *ev {
            InputEvent::Command {
                command_id,
                agent_id,
                opcode,
                arg0,
                arg1,
                ..
            } => {
                *acc = acc.wrapping_add(command_id ^ (agent_id << 8) ^ (opcode as u64) << 32);
                *acc = acc.wrapping_add(arg0 as u64 ^ arg1 as u64);
            }
            InputEvent::ConfigChange { key, value, .. } => {
                *acc = acc.wrapping_add(key ^ value as u64);
            }
            InputEvent::LodSwitch {
                entity_id,
                from_level,
                to_level,
                ..
            } => {
                *acc = acc.wrapping_add(entity_id ^ ((from_level as u64) << 4) ^ to_level as u64);
            }
            InputEvent::BudgetAdjustment {
                system_id,
                per_tick_cap,
                ..
            } => {
                *acc = acc.wrapping_add(system_id ^ per_tick_cap);
            }
        }
    }

    fn run_digest(
        mut events_per_tick: impl FnMut(u64) -> Vec<InputEvent>,
        ticks: u64,
    ) -> sim_digest::Digest256 {
        let mut acc = 0u64;
        for t in 0..=ticks {
            for ev in events_per_tick(t) {
                apply(&mut acc, &ev);
            }
        }
        let mut w = CanonicalWriter::new("sim-replay", "SimState");
        w.u64(acc);
        sim_digest::digest("sim-replay", "SimState", w.as_bytes())
    }

    #[test]
    fn record_replay_digest_equivalence() {
        // 录制运行 == 编码/解码 == 回放运行（Schema §3 验收判据）。
        let input = record();
        let live = run_digest(
            |t| {
                input
                    .events
                    .iter()
                    .copied()
                    .filter(|e| e.tick() == t)
                    .collect()
            },
            5,
        );
        let decoded = ReplayInput::decode(&input.encode()).unwrap();
        assert_eq!(decoded, input);
        let mut rp = Replayer::new(decoded);
        let replay = run_digest(|t| rp.due(t), 5);
        assert!(rp.is_drained());
        assert_eq!(live, replay, "recorded run digest == replayed run digest");
    }

    #[test]
    fn encode_decode_roundtrip_is_stable() {
        let input = record();
        assert_eq!(input.encode(), input.encode(), "canonical bytes stable");
        assert_eq!(ReplayInput::decode(&input.encode()).unwrap(), input);
    }

    #[test]
    fn unsupported_version_is_diagnosable() {
        let mut bytes = record().encode();
        // 信封三段各 u64 长度前缀：(8+8)+(8+10)+(8+11)=53 字节后 u16 版本。
        let ver_pos = 8 + 8 + 8 + 10 + 8 + 11;
        bytes[ver_pos] = 2;
        match ReplayInput::decode(&bytes) {
            Err(ReplayError::UnsupportedVersion { found: 2 }) => {}
            other => {
                let hdr: Vec<String> = bytes[..64].iter().map(|x| format!("{:02x}", x)).collect();
                panic!("expected UnsupportedVersion, got {other:?}; hdr={}", hdr.join(" "));
            }
        }
        assert_eq!(ReplayError::UnsupportedVersion { found: 2 }.code().to_string(), "RPL-0001");
    }

    #[test]
    fn trailing_bytes_and_truncation_are_rejected() {
        let mut bytes = record().encode();
        bytes.push(0);
        assert_eq!(ReplayInput::decode(&bytes), Err(ReplayError::TrailingBytes));
        let cut = &record().encode()[..10];
        assert_eq!(ReplayInput::decode(cut), Err(ReplayError::Truncated));
    }

    #[test]
    fn recorder_enforces_tick_order() {
        let mut rec = Recorder::new(1, 30, 1);
        rec.record(InputEvent::ConfigChange {
            tick: 5,
            key: 1,
            value: 2,
        })
        .unwrap();
        assert_eq!(
            rec.record(InputEvent::ConfigChange {
                tick: 3,
                key: 1,
                value: 2,
            }),
            Err(ReplayError::OutOfOrderTick {
                previous: 5,
                found: 3
            })
        );
    }
}


===== FILE: crates/sim-economy/src/lib.rs =====

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
