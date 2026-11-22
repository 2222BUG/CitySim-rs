# STAGE2B3-BUNDLE PART 12（本卷文件：crates/sim-digest/src/lib.rs, crates/sim-core/src/lib.rs）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3；与 ZIP 同源；粘贴时请整卷一条消息发送 -->


===== FILE: crates/sim-digest/src/lib.rs =====

//! CitySim-rs sim-digest（Stage 1 T6）：GlobalDigest 权威状态摘要（ADR-NEW-19）。
//!
//! ADR-NEW-19 决策（2026-11-19 正式裁决采纳，附条件）：
//! - 哈希：BLAKE3-256（备选 xxh3-64；否决 SHA-256/CRC64/serde 默认）。
//! - 编码：与存档共用底层 canonical 编码规则（sim-codec），顶层 Schema 相互
//!   独立（本 crate 的信封 Schema 仅用于摘要），domain separation 由
//!   sim-codec 头（编码规则版本 | 模块标识 | 类型标识）承担。
//! - digest_version u16：算法/纳入范围/编码变更必须 bump。
//! - 存档点全量重算为判据；每 tick 滚动摘要仅是优化（保留与否以实测为准，
//!   条件核销清单未决项）。
//! - 纳入范围=全部权威状态（含预算队列/让步状态，NEW-19 条件③）——由调用方
//!   按 `record(module, kind, payload)` 逐模块喂入；模块清单随 T7/T9 接入补全。
//!
//! 证据纪律：BLAKE3 跨平台位级一致性（x86_64/aarch64）为条件核销清单未决项
//! （ADR-NEW-19 条件①，挂 R4）；黄金向量见 docs/benchmarks/globaldigest-golden-vectors.txt。
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use sim_codec::CanonicalWriter;

/// 摘要协议版本（EC-03 存档字段口径）。算法/纳入范围/编码变更必须 bump。
pub const DIGEST_VERSION: u16 = 1;

/// 256 位摘要（BLAKE3-256 输出）。
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Digest256(pub [u8; 32]);

impl Digest256 {
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl std::fmt::Debug for Digest256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Digest256({})", self.hex())
    }
}

/// 单条状态摘要：envelope = codec 头(编码规则版本|module|kind)
///   | digest_version u16 | payload(u64 长度前缀) → BLAKE3-256。
pub fn digest(module: &str, kind: &str, payload: &[u8]) -> Digest256 {
    let mut w = CanonicalWriter::new(module, kind);
    w.u16(DIGEST_VERSION);
    w.bytes(payload);
    Digest256(*blake3::hash(w.as_bytes()).as_bytes())
}

/// GlobalDigest：权威状态总摘要。
/// 记录以 (module, kind) 为键的模块级摘要，finalize 时按键序（BTreeMap 稳定序）
/// 合成总摘要——**记录顺序无关**（2.9 条 4 口径）。
#[derive(Default)]
pub struct GlobalDigest {
    entries: BTreeMap<(String, String), [u8; 32]>,
}

impl GlobalDigest {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一个模块的一类权威状态（重复记录同键=覆盖，取最后一次）。
    pub fn record(&mut self, module: &str, kind: &str, payload: &[u8]) {
        self.entries
            .insert((module.to_string(), kind.to_string()), digest(module, kind, payload).0);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 合成总摘要：envelope | 条目数 | (键, 摘要)* 按键序 → BLAKE3-256。
    pub fn finalize(&self) -> Digest256 {
        let mut w = CanonicalWriter::new("sim-digest", "GlobalDigest");
        w.u16(DIGEST_VERSION);
        w.seq_len(self.entries.len());
        for ((module, kind), hash) in &self.entries {
            w.str(module);
            w.str(kind);
            w.bytes(hash);
        }
        Digest256(*blake3::hash(w.as_bytes()).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_version_pinned() {
        assert_eq!(DIGEST_VERSION, 1, "摘要协议变更必须 bump 并修订 ADR-NEW-19");
    }

    /// domain separation：模块/类型标识参与信封，同负载不同域摘要不同。
    #[test]
    fn domain_separation() {
        let a = digest("sim-agent", "AgentState", b"payload");
        let b = digest("sim-network", "AgentState", b"payload");
        let c = digest("sim-agent", "EdgeState", b"payload");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    /// 负载敏感性：单字节变化必须改变摘要。
    #[test]
    fn payload_sensitivity() {
        let a = digest("m", "k", b"payload-x");
        let b = digest("m", "k", b"payload-y");
        assert_ne!(a, b);
    }

    /// 同输入稳定（同进程两次调用一致；跨构建稳定性由黄金向量测试保证）。
    #[test]
    fn same_input_same_digest() {
        assert_eq!(digest("m", "k", b"x"), digest("m", "k", b"x"));
    }

    /// 记录顺序无关：BTreeMap 键序合成。
    #[test]
    fn global_digest_order_independent() {
        let mut g1 = GlobalDigest::new();
        g1.record("sim-agent", "Agents", b"a1");
        g1.record("sim-network", "Network", b"n1");
        g1.record("sim-economy", "Economy", b"e1");
        let mut g2 = GlobalDigest::new();
        g2.record("sim-economy", "Economy", b"e1");
        g2.record("sim-agent", "Agents", b"a1");
        g2.record("sim-network", "Network", b"n1");
        assert_eq!(g1.finalize(), g2.finalize());
    }

    /// 模块敏感性：任一模块状态变化必须改变总摘要。
    #[test]
    fn global_digest_module_sensitivity() {
        let mut g1 = GlobalDigest::new();
        g1.record("sim-agent", "Agents", b"a1");
        g1.record("sim-network", "Network", b"n1");
        let mut g2 = GlobalDigest::new();
        g2.record("sim-agent", "Agents", b"a2");
        g2.record("sim-network", "Network", b"n1");
        assert_ne!(g1.finalize(), g2.finalize());
    }

    /// 空集合确定性：无记录时总摘要稳定。
    #[test]
    fn empty_global_digest_is_stable() {
        assert_eq!(GlobalDigest::new().finalize(), GlobalDigest::new().finalize());
    }

    /// 黄金向量打印（生成后固化进 golden_vectors 测试与
    /// docs/benchmarks/globaldigest-golden-vectors.txt）：
    /// cargo test -p sim-digest print_goldens -- --nocapture
    #[test]
    fn print_goldens() {
        println!("DIGEST_VERSION = {DIGEST_VERSION}");
        println!(
            r#"digest("sim-agent","Agents",b"golden-1") = {}"#,
            digest("sim-agent", "Agents", b"golden-1").hex()
        );
        println!(
            r#"digest("sim-digest","GlobalDigest",b"") = {}"#,
            digest("sim-digest", "GlobalDigest", b"").hex()
        );
        let mut g = GlobalDigest::new();
        g.record("sim-agent", "Agents", b"a1");
        g.record("sim-network", "Network", b"n1");
        println!("global(a1,n1) = {}", g.finalize().hex());
    }

    /// 黄金向量（跨构建稳定性=R2 oracle 基础）：2026-11-19 print_goldens 打印后固化；
    /// 向量文件 docs/benchmarks/globaldigest-golden-vectors.txt。
    /// 本测试失败=摘要协议/编码规则变更未 bump digest_version。
    #[test]
    fn golden_vectors() {
        assert_eq!(
            digest("sim-agent", "Agents", b"golden-1").hex(),
            "64ec5fc5c6062bc58af4df5050df8920e8f79201bfbe15d61ac1ef2287d15b2f"
        );
        assert_eq!(
            digest("sim-digest", "GlobalDigest", b"").hex(),
            "0f677f5692aaf571f6d440c1f931722ff5de1d99965baaec48472389f5091bb5"
        );
        let mut g = GlobalDigest::new();
        g.record("sim-agent", "Agents", b"a1");
        g.record("sim-network", "Network", b"n1");
        assert_eq!(
            g.finalize().hex(),
            "11d8b5e30bed8a3b9f1bb17c0d5ebb71f513b0d73a794d0c6e2f970d39f1d1d5"
        );
    }
}


===== FILE: crates/sim-core/src/lib.rs =====

//! 模拟核心：管理模拟状态和 Tick 推进（主文档 2.2 架构分层）。
//! 权威crate。
//!
//! Stage 1 T1 骨架（2026-11-19 建立）——任务清单见 docs/architecture/Stage-1-Task-List.md。
//! 依赖方向约束见 docs/architecture/crate-authority-list.md 与 .github/scripts/dep_graph_check.py。
//!
//! Stage 1 T3（2026-11-19）：权威时间与玩家命令日志——
//! u64 tick（2.9 条 7：不读系统时钟）、命令带生效 tick 并记录供回放（2.9 条 8）。
//! 命令 ID 分配器为 u64 单调高水位口径（ADR-NEW-20：高水位入存档防 ID 复用）。
//! Stage 2C C3a（2026-11-19）：预算注册表与确定性降级（budget 模块，ADR-NEW-14 C2 细化）。

pub mod budget;

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "模拟核心";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

/// 权威时间：u64 tick（2.9 条 7）。存档点、命令生效点、预算调整、LOD 切换
/// 共同以 tick 边界为界（2.8）。本类型不含任何时钟读取路径。
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimClock {
    tick: u64,
}

impl SimClock {
    pub const fn new() -> Self {
        Self { tick: 0 }
    }

    pub const fn tick(self) -> u64 {
        self.tick
    }

    /// tick 边界推进（单 tick 递增；唯一的时间前进入口）。
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }
}

/// 命令 ID 分配器：u64 单调递增，高水位入存档（ADR-NEW-20）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandIdAllocator {
    high_water: u64,
}

impl CommandIdAllocator {
    pub const fn new() -> Self {
        Self { high_water: 0 }
    }

    /// 分配下一个命令 ID（单调递增，不复用）。
    pub fn alloc(&mut self) -> u64 {
        self.high_water += 1;
        self.high_water
    }

    /// 已分配的最大 ID（=存档最小集中的高水位字段）。
    pub const fn high_water(self) -> u64 {
        self.high_water
    }

    /// 读档恢复高水位（必须 ≥ 当前值，防 ID 复用——EC-03/ADR-NEW-20）。
    pub fn restore(&mut self, high_water: u64) {
        assert!(
            high_water >= self.high_water,
            "高水位回退会导致命令 ID 复用"
        );
        self.high_water = high_water;
    }
}

/// 玩家命令载荷（权威侧整数坐标：1mm 网格，2.10 定案口径；无浮点）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandPayload {
    /// 移动到坐标（网格单位 1mm）。
    MoveTo { x: i64, y: i64 },
    /// 前往路网节点。
    SetDestination { node: u64 },
    /// 取消在途寻路请求。
    CancelRequest { request_id: u64 },
}

/// 玩家命令：带生效 tick 并完整记录供回放（2.9 条 8）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PlayerCommand {
    pub command_id: u64,
    /// 生效 tick（不得早于提交时的当前 tick）。
    pub effective_tick: u64,
    pub agent_id: u64,
    pub payload: CommandPayload,
}

/// 提交命令的校验错误。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SubmitError {
    /// 生效 tick 早于当前 tick：命令不得在"过去"生效。
    EffectiveTickInPast,
}

/// 命令日志：确定性存储 + 回放输入（EC-03 最小集/2.9 条 8）。
/// 到期查询输出全序 = (effective_tick, command_id)（2.9 条 4：不依赖到达顺序）。
#[derive(Clone, Debug, Default)]
pub struct CommandLog {
    commands: Vec<PlayerCommand>,
    alloc: CommandIdAllocator,
}

impl CommandLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// 提交命令：分配 ID、校验生效 tick、记录。
    pub fn submit(
        &mut self,
        agent_id: u64,
        effective_tick: u64,
        current_tick: u64,
        payload: CommandPayload,
    ) -> Result<u64, SubmitError> {
        if effective_tick < current_tick {
            return Err(SubmitError::EffectiveTickInPast);
        }
        let command_id = self.alloc.alloc();
        self.commands.push(PlayerCommand {
            command_id,
            effective_tick,
            agent_id,
            payload,
        });
        Ok(command_id)
    }

    /// 截至 tick 已生效的命令，按 (effective_tick, command_id) 全序返回
    /// （确定性：与提交到达顺序无关）。
    pub fn due_commands(&self, tick: u64) -> Vec<PlayerCommand> {
        let mut due: Vec<PlayerCommand> = self
            .commands
            .iter()
            .copied()
            .filter(|c| c.effective_tick <= tick)
            .collect();
        due.sort_by_key(|c| (c.effective_tick, c.command_id));
        due
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 命令 ID 高水位（入存档字段）。
    pub const fn id_high_water(&self) -> u64 {
        self.alloc.high_water
    }

    /// 读档/回放重建入口：从已记录命令与 ID 高水位重建日志（EC-03 保存并恢复）。
    pub fn from_parts(commands: Vec<PlayerCommand>, id_high_water: u64) -> Self {
        Self {
            commands,
            alloc: CommandIdAllocator {
                high_water: id_high_water,
            },
        }
    }

    /// 读档恢复高水位（防 ID 复用）。
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

    /// 权威时间：单 tick 推进，确定性。
    #[test]
    fn clock_advances_by_one_at_tick_boundary() {
        let mut clock = SimClock::new();
        assert_eq!(clock.tick(), 0);
        clock.advance();
        clock.advance();
        assert_eq!(clock.tick(), 2);
    }

    /// 命令带生效 tick（2.9 条 8）：字段完整。
    #[test]
    fn command_carries_effective_tick_and_id() {
        let mut log = CommandLog::new();
        let id = log
            .submit(7, 10, 5, CommandPayload::MoveTo { x: 1000, y: -2000 })
            .unwrap();
        let due = log.due_commands(10);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].command_id, id);
        assert_eq!(due[0].effective_tick, 10);
        assert_eq!(due[0].agent_id, 7);
    }

    /// 生效 tick 不得在过去。
    #[test]
    fn past_effective_tick_is_rejected() {
        let mut log = CommandLog::new();
        assert_eq!(
            log.submit(1, 3, 4, CommandPayload::SetDestination { node: 9 }),
            Err(SubmitError::EffectiveTickInPast)
        );
        assert!(log.is_empty());
    }

    /// 到期全序=(effective_tick, command_id)：同一命令集（含相同 ID）无论物理
    /// 存储顺序如何，输出一致（2.9 条 4：不依赖到达/迭代顺序）。
    #[test]
    fn due_order_is_stable_regardless_of_storage_order() {
        let mk = |id, tick, agent| PlayerCommand {
            command_id: id,
            effective_tick: tick,
            agent_id: agent,
            payload: CommandPayload::MoveTo { x: 0, y: 0 },
        };
        let cmds = vec![mk(1, 5, 1), mk(2, 6, 2), mk(3, 5, 3)];
        let mut reversed = cmds.clone();
        reversed.reverse();
        let a = CommandLog::from_parts(cmds, 3);
        let b = CommandLog::from_parts(reversed, 3);
        let da = a.due_commands(6);
        let db = b.due_commands(6);
        assert_eq!(da, db);
        let keys: Vec<(u64, u64)> = da.iter().map(|c| (c.effective_tick, c.command_id)).collect();
        assert_eq!(keys, vec![(5, 1), (5, 3), (6, 2)]);
    }

    /// 命令 ID 单调不复用；高水位恢复防复用（ADR-NEW-20）。
    #[test]
    fn id_high_water_prevents_reuse() {
        let mut log = CommandLog::new();
        let id1 = log.submit(1, 1, 0, CommandPayload::MoveTo { x: 0, y: 0 }).unwrap();
        let id2 = log.submit(1, 1, 0, CommandPayload::MoveTo { x: 0, y: 0 }).unwrap();
        assert_eq!((id1, id2), (1, 2));
        assert_eq!(log.id_high_water(), 2);
        // 模拟读档：新日志恢复高水位后，新分配从 3 起。
        let mut restored = CommandLog::new();
        restored.restore_id_high_water(log.id_high_water());
        let id3 = restored.submit(1, 1, 0, CommandPayload::MoveTo { x: 0, y: 0 }).unwrap();
        assert_eq!(id3, 3);
    }

    /// 高水位回退=ID 复用风险，必须拒绝（panic 路径）。
    #[test]
    #[should_panic(expected = "高水位回退会导致命令 ID 复用")]
    fn high_water_must_not_go_backwards() {
        let mut alloc = CommandIdAllocator::new();
        alloc.alloc();
        alloc.alloc();
        alloc.restore(1);
    }
}
