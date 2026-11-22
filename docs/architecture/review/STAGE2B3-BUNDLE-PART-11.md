# STAGE2B3-BUNDLE PART 11（本卷文件：crates/sim-test-support/src/deterministic.rs, crates/sim-test-support/src/property.rs, crates/sim-test-support/src/benchmark.rs, crates/sim-test-support/src/assertions.rs, crates/sim-error/src/lib.rs, crates/sim-codec/src/lib.rs）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3；与 ZIP 同源；粘贴时请整卷一条消息发送 -->


===== FILE: crates/sim-test-support/src/deterministic.rs =====

//! 确定性黄金测试 + 读档等价测试模板（EC-07 deterministic）。
//!
//! 模板来源：docs/benchmarks/SaveRestore-Validation-REPORT.md（Step 5 已实证
//! 强/弱两种形式）。任何实现 [`SimHarness`] 的模拟核心都可直接套用。
//!
//! 确定性测试口径（EC-07）：固定输入和种子，重复运行并比较摘要；
//! 覆盖线程数无关性（R3 每日构建）与读档等价性（R2 门禁，强形式）。

/// 模拟器测试接口。
pub trait SimHarness {
    /// 运行到指定 tick，返回该 tick 的状态摘要。
    fn run_to(&mut self, tick: u64) -> u64;
    /// 存档（仅 tick 边界语义由实现保证，EC-03）。
    fn save(&self) -> Vec<u8>;
    /// 读档。
    fn load(&mut self, blob: &[u8]);
}

/// 强读档等价（R2 强形式，EC-11）：
/// - run_a：直接跑到 M，取摘要 d_a；
/// - run_b：跑到 N（N < M）并存档；
/// - run_c：新实例载入同一存档，跑到 M，取摘要 d_c；
/// - 断言由调用方完成：d_a == d_c。
///
/// 依赖在途任务"保存并恢复"语义（ADR-NEW-16 定案）；若政策改为
/// "丢弃并重新入队"，必须改用 [`weak_form_digests`] 并另行 ADR 修订。
pub fn strong_readback_digests<H: SimHarness>(
    make: impl Fn() -> H,
    n: u64,
    m: u64,
) -> (u64, u64) {
    assert!(n < m, "save point N must precede target tick M");
    let mut a = make();
    let d_a = a.run_to(m);
    let mut b = make();
    b.run_to(n);
    let blob = b.save();
    let mut c = make();
    c.load(&blob);
    let d_c = c.run_to(m);
    (d_a, d_c)
}

/// 弱形式等价（政策改"丢弃并重新入队"时的降级口径）：
/// 两次独立的 save/load 运行之间摘要一致。
pub fn weak_form_digests<H: SimHarness>(make: impl Fn() -> H, n: u64, m: u64) -> (u64, u64) {
    assert!(n < m, "save point N must precede target tick M");
    let run = |fresh: bool| -> u64 {
        let mut b = make();
        b.run_to(n);
        let blob = b.save();
        let mut c = make();
        c.load(&blob);
        let _ = fresh;
        c.run_to(m)
    };
    (run(true), run(false))
}

/// 黄金值断言辅助：固定输入重复运行两次，摘要必须一致。
pub fn repeat_run_digests<H: SimHarness>(make: impl Fn() -> H, m: u64) -> (u64, u64) {
    let mut a = make();
    let d1 = a.run_to(m);
    let mut b = make();
    let d2 = b.run_to(m);
    (d1, d2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 玩具计数器模拟器：演示模板自检（非真实模拟核心）。
    struct CounterSim {
        tick: u64,
        acc: u64,
    }

    const MULT: u64 = 6_364_136_223_846_793_005;
    const INC: u64 = 1_442_695_040_888_963_407;

    impl CounterSim {
        fn new() -> CounterSim {
            CounterSim { tick: 0, acc: 0x5EED }
        }
    }

    impl SimHarness for CounterSim {
        fn run_to(&mut self, tick: u64) -> u64 {
            while self.tick < tick {
                self.tick += 1;
                self.acc = self.acc.wrapping_mul(MULT).wrapping_add(INC ^ self.tick);
            }
            self.acc ^ self.tick
        }
        fn save(&self) -> Vec<u8> {
            let mut v = Vec::new();
            v.extend_from_slice(&self.tick.to_le_bytes());
            v.extend_from_slice(&self.acc.to_le_bytes());
            v
        }
        fn load(&mut self, blob: &[u8]) {
            assert_eq!(blob.len(), 16, "toy save must be 16 bytes");
            self.tick = u64::from_le_bytes(blob[0..8].try_into().unwrap());
            self.acc = u64::from_le_bytes(blob[8..16].try_into().unwrap());
        }
    }

    #[test]
    fn strong_readback_template_works_on_mock() {
        let (d_a, d_c) = strong_readback_digests(CounterSim::new, 40, 100);
        assert_eq!(d_a, d_c, "strong readback equivalence must hold on mock");
    }

    #[test]
    fn weak_form_template_works_on_mock() {
        let (d1, d2) = weak_form_digests(CounterSim::new, 40, 100);
        assert_eq!(d1, d2);
    }

    #[test]
    fn repeat_run_is_deterministic_on_mock() {
        let (d1, d2) = repeat_run_digests(CounterSim::new, 77);
        assert_eq!(d1, d2);
    }
}


===== FILE: crates/sim-test-support/src/property.rs =====

//! 属性测试不变量助手（EC-07 property）。
//!
//! 对应 EC-07 不变量清单：
//! 1. 路网边引用合法；
//! 2. Agent 身份不重复；
//! 3. 事务发布前后状态有效；
//! 4. 数值运算遵守预期边界；
//! 5. 过期寻路结果的重算次数不超过上限。
//!
//! 本模块只提供不变量判定函数；随机输入由调用方以固定种子生成
//! （确定性属性测试，2.9 口径）。

/// 不变量 1：路网边引用合法（端点 < node_count，cost 为正）。
pub fn edges_reference_valid_nodes(edges: &[(u64, u64, u64)], node_count: u64) -> bool {
    edges.iter().all(|&(a, b, cost)| a < node_count && b < node_count && a != b && cost > 0)
}

/// 不变量 2：身份不重复（AgentId / 请求 ID / 事务 ID 通用）。
pub fn ids_unique(ids: &[u64]) -> bool {
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    sorted.windows(2).all(|w| w[0] != w[1])
}

/// 不变量 3：事务发布前后状态有效——版本严格单调递增且只前进一格
/// （原子发布，ADR-NEW-17；失败不得改版本，Spike C 已实证）。
pub fn tx_publish_valid(before_version: u64, after_version: u64, publish_ok: bool) -> bool {
    if publish_ok {
        after_version == before_version + 1
    } else {
        after_version == before_version
    }
}

/// 不变量 4：数值运算遵守预期边界（SimScalar / 中间值位宽，ADR-NEW-21）。
pub fn within_bounds(value: i128, min: i128, max: i128) -> bool {
    value >= min && value <= max
}

/// 不变量 5：过期寻路结果的重算次数不超过上限（ADR-NEW-17 / ADR-NEW-23：
/// recompute_limit，超限进入兜底而非继续重算）。
pub fn recompute_within_limit(count: u64, limit: u64) -> bool {
    count <= limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariants_hold_on_valid_inputs() {
        assert!(edges_reference_valid_nodes(&[(0, 1, 100), (1, 2, 100)], 3));
        assert!(!edges_reference_valid_nodes(&[(0, 1, 100), (1, 9, 100)], 3), "dangling node");
        assert!(!edges_reference_valid_nodes(&[(0, 1, 0)], 2), "zero cost");

        assert!(ids_unique(&[3, 1, 2]));
        assert!(!ids_unique(&[1, 1]));

        assert!(tx_publish_valid(7, 8, true));
        assert!(!tx_publish_valid(7, 9, true), "version must advance by one");
        assert!(tx_publish_valid(7, 7, false), "failed tx must not change version");

        assert!(within_bounds(0, i64::MIN as i128, i64::MAX as i128));
        assert!(!within_bounds(i64::MAX as i128 + 1, i64::MIN as i128, i64::MAX as i128));

        assert!(recompute_within_limit(3, 3));
        assert!(!recompute_within_limit(4, 3));
    }
}


===== FILE: crates/sim-test-support/src/benchmark.rs =====

//! 基准测试模板（EC-07 benchmark）：固定场景、硬件、构建模式和 Tick 数，
//! 记录运行耗时与内存占用。
//!
//! 口径：本模块属测试/遥测侧，允许墙钟（2.9 只约束权威模拟系统）；
//! 正式基准记录统一存 docs/benchmarks/（证据纪律）。

/// 一次基准运行的记录。
#[derive(Clone, Debug, PartialEq)]
pub struct BenchRecord {
    pub scenario: String,
    pub hardware_note: String,
    pub build_mode: String,
    pub ticks: u64,
    pub wall_ms: f64,
    pub memory_note: String,
}

impl BenchRecord {
    /// 汇报行（供 docs/benchmarks/ 归档）。
    pub fn to_report_line(&self) -> String {
        format!(
            "{} | {} | {} | {} ticks | {:.3} ms | {}",
            self.scenario, self.hardware_note, self.build_mode, self.ticks, self.wall_ms,
            self.memory_note
        )
    }
}

/// 构造一条基准记录。
pub fn record(
    scenario: &str,
    hardware_note: &str,
    build_mode: &str,
    ticks: u64,
    wall_ms: f64,
    memory_note: &str,
) -> BenchRecord {
    BenchRecord {
        scenario: scenario.to_string(),
        hardware_note: hardware_note.to_string(),
        build_mode: build_mode.to_string(),
        ticks,
        wall_ms,
        memory_note: memory_note.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_line_contains_required_fields() {
        let r = record("r2-gate-10k", "win-x86_64 rustc1.100", "release", 1_000, 1234.5, "n/a");
        let line = r.to_report_line();
        assert!(line.contains("r2-gate-10k"));
        assert!(line.contains("release"));
        assert!(line.contains("1000 ticks"));
        assert!(line.contains("1234.500 ms"));
    }
}


===== FILE: crates/sim-test-support/src/assertions.rs =====

//! 模拟状态断言（EC-07 assertions）。

/// 断言：摘要一致（确定性回归 / 读档等价通用）。
#[track_caller]
pub fn assert_digest_eq(context: &str, d1: u64, d2: u64) {
    assert_eq!(d1, d2, "digest mismatch in {}: {:#018x} != {:#018x}", context, d1, d2);
}

/// 断言：序列按稳定键升序（2.9 条 4——并行提交/查询结果按稳定键排序）。
#[track_caller]
pub fn assert_sorted_by_key(keys: &[u64]) {
    for w in keys.windows(2) {
        assert!(w[0] <= w[1], "keys not in stable order: {} > {}", w[0], w[1]);
    }
}

/// 断言：读档后高水位不得回退（ID 不复用，ADR-NEW-20 / EC-03 身份类）。
#[track_caller]
pub fn assert_high_water_non_decreasing(context: &str, before: u64, after: u64) {
    assert!(after >= before, "high water regressed in {}: {} -> {}", context, before, after);
}

/// 断言：tick 边界语义（存档/预算调整/命令生效仅发生在 tick 边界）。
#[track_caller]
pub fn assert_tick_boundary(context: &str, current_tick: u64, boundary_tick: u64) {
    assert_eq!(
        current_tick, boundary_tick,
        "{} must occur at a tick boundary: current {} != boundary {}",
        context, current_tick, boundary_tick
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertions_pass_on_valid_state() {
        assert_digest_eq("mock", 1, 1);
        assert_sorted_by_key(&[1, 1, 2, 3]);
        assert_high_water_non_decreasing("alloc", 5, 5);
        assert_high_water_non_decreasing("alloc", 5, 6);
        assert_tick_boundary("save", 40, 40);
    }

    #[test]
    #[should_panic(expected = "digest mismatch")]
    fn digest_mismatch_panics() {
        assert_digest_eq("mock", 1, 2);
    }
}


===== FILE: crates/sim-error/src/lib.rs =====

//! CitySim-rs sim-error（Stage 1 T8）：错误分层骨架（EC-08）。
//!
//! EC-08 分层：Fatal（不可恢复，必须停机/拒绝加载）/ Recoverable（可继续模拟，
//! 有确定性处置）/ Diagnostic（诊断信息，不改变权威状态）。
//! SimErrorContext.entity_id 语义 = AgentId（2.3：与 Bevy Entity 解耦的稳定
//! u64 身份；正式类型随 T4 在 sim-agent 定义，此处以 u64 表达并以文档钉语义）。
//!
//! 错误码规范见 docs/architecture/error-code-spec.md（格式 MOD-NNNN）。
//! 诊断事件通道为确定性 Vec（产生顺序=drain 顺序），无 HashMap/时钟/浮点。
#![forbid(unsafe_code)]

/// 错误分层（EC-08）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    /// 不可恢复：存档损坏、协议版本不兼容、不变量破坏——必须终止本次运行。
    Fatal,
    /// 可恢复：确定性处置后模拟可继续（如过期寻路结果重入队）。
    Recoverable,
    /// 诊断：不改变权威状态（如预算逼近上限的提示）。
    Diagnostic,
}

/// 错误码：模块前缀 + 4 位编号（规范：docs/architecture/error-code-spec.md）。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    pub module: &'static str,
    pub number: u32,
}

impl ErrorCode {
    pub const fn new(module: &'static str, number: u32) -> Self {
        Self { module, number }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:04}", self.module, self.number)
    }
}

/// 错误上下文（确定性：tick 为 u64，无墙钟；entity_id 语义=AgentId）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SimErrorContext {
    /// 权威时间：u64 tick（2.9 条 7；不读系统时钟）。
    pub tick: u64,
    /// 相关实体的稳定身份。语义 = AgentId（2.3，与 Bevy Entity 解耦）。
    pub entity_id: u64,
    /// 产生点的系统标识（namespace 口径，如 "sim-pathfinding"）。
    pub system: &'static str,
}

/// 分层错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimError {
    pub class: ErrorClass,
    pub code: ErrorCode,
    pub message: &'static str,
    pub context: SimErrorContext,
}

impl SimError {
    pub const fn new(
        class: ErrorClass,
        code: ErrorCode,
        message: &'static str,
        context: SimErrorContext,
    ) -> Self {
        Self {
            class,
            code,
            message,
            context,
        }
    }
}

/// 诊断事件（Diagnostic 层的结构化记录；不改变权威状态）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticEvent {
    pub code: ErrorCode,
    pub message: &'static str,
    pub context: SimErrorContext,
}

/// 诊断事件通道：确定性 Vec，emit 顺序 = drain 顺序（2.9 条 4 口径：
/// 不依赖任何无序迭代；事件可作为 GlobalDigest 输入在 T9 接入）。
#[derive(Default)]
pub struct DiagnosticSink {
    events: Vec<DiagnosticEvent>,
}

impl DiagnosticSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, code: ErrorCode, message: &'static str, context: SimErrorContext) {
        self.events.push(DiagnosticEvent {
            code,
            message,
            context,
        });
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// 按产生顺序取出全部事件（清空通道）。
    pub fn drain(&mut self) -> Vec<DiagnosticEvent> {
        std::mem::take(&mut self.events)
    }
}

/// 常用错误码登记（增量登记规范见 error-code-spec.md；编号不得复用）。
pub mod codes {
    use super::ErrorCode;

    /// 存档损坏/一致性检查失败（Fatal）。
    pub const SAVE_CORRUPT: ErrorCode = ErrorCode::new("SAV", 1);
    /// 存档 Schema 版本不兼容（Fatal，可诊断拒绝加载）。
    pub const SAVE_SCHEMA_UNSUPPORTED: ErrorCode = ErrorCode::new("SAV", 2);
    /// 过期寻路结果重入队（Recoverable）。
    pub const PATH_STALE_REQUEUE: ErrorCode = ErrorCode::new("PTH", 1);
    /// 重算超限进入受阻兜底（Recoverable）。
    pub const PATH_FALLBACK_BLOCKED: ErrorCode = ErrorCode::new("PTH", 2);
    /// 预算逼近上限提示（Diagnostic）。
    pub const BUDGET_PRESSURE: ErrorCode = ErrorCode::new("BGT", 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> SimErrorContext {
        SimErrorContext {
            tick: 42,
            entity_id: 7,
            system: "sim-pathfinding",
        }
    }

    /// 分层可表达三类。
    #[test]
    fn three_classes_exist() {
        let e = SimError::new(ErrorClass::Fatal, codes::SAVE_CORRUPT, "save digest mismatch", ctx());
        assert_eq!(e.class, ErrorClass::Fatal);
        assert_eq!(
            SimError::new(ErrorClass::Recoverable, codes::PATH_STALE_REQUEUE, "stale", ctx()).class,
            ErrorClass::Recoverable
        );
        assert_eq!(
            SimError::new(ErrorClass::Diagnostic, codes::BUDGET_PRESSURE, "pressure", ctx()).class,
            ErrorClass::Diagnostic
        );
    }

    /// 错误码格式 MOD-NNNN。
    #[test]
    fn error_code_format() {
        assert_eq!(codes::SAVE_CORRUPT.to_string(), "SAV-0001");
        assert_eq!(codes::PATH_FALLBACK_BLOCKED.to_string(), "PTH-0002");
        assert_eq!(codes::BUDGET_PRESSURE.to_string(), "BGT-0001");
    }

    /// 登记码唯一（模块+编号不重复）。
    #[test]
    fn registered_codes_are_unique() {
        let all = [
            codes::SAVE_CORRUPT,
            codes::SAVE_SCHEMA_UNSUPPORTED,
            codes::PATH_STALE_REQUEUE,
            codes::PATH_FALLBACK_BLOCKED,
            codes::BUDGET_PRESSURE,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b, "错误码不得重复登记");
            }
        }
    }

    /// 上下文：tick 为 u64、entity_id 语义=AgentId（文档钉义，此处断言字段可表达）。
    #[test]
    fn context_fields() {
        let c = ctx();
        let tick: u64 = c.tick;
        let agent: u64 = c.entity_id;
        assert_eq!((tick, agent), (42, 7));
    }

    /// 诊断通道确定性：emit 顺序=drain 顺序，drain 后清空。
    #[test]
    fn diagnostic_sink_is_ordered_and_drains() {
        let mut sink = DiagnosticSink::new();
        sink.emit(codes::BUDGET_PRESSURE, "p1", ctx());
        sink.emit(codes::BUDGET_PRESSURE, "p2", ctx());
        assert_eq!(sink.len(), 2);
        let events = sink.drain();
        assert_eq!(events[0].message, "p1");
        assert_eq!(events[1].message, "p2");
        assert!(sink.is_empty());
    }
}


===== FILE: crates/sim-codec/src/lib.rs =====

//! CitySim-rs sim-codec（Stage 1 T6）：canonical 序列化编码器（ADR-NEW-19）。
//!
//! 存档与 GlobalDigest 共用**底层编码规则**（字段编码、端序、可选值、字段顺序、
//! 稳定身份编码、版本标识），但**顶层 Schema 相互独立**（存档/摘要各自版本化、
//! 迁移逻辑独立）——P0-04 修订口径。
//!
//! 编码规则 v1：
//! - 整数一律小端定长（u8/u16/u32/u64/i64）；usize 以 u64 编码。
//! - 变长数据 = u64 长度前缀 + 原始字节；字符串 = UTF-8 字节同上。
//! - 浮点禁入（2.9 条 3；权威侧无 f32/f64）。
//! - domain separation：编码流首部含编码规则版本、模块标识、类型标识
//!   （ADR-NEW-19 条 2）。
//! - 字段顺序即编码顺序（调用方约定，由各 Schema 文档固定）。
#![forbid(unsafe_code)]

/// 编码规则版本标识（变更必须 bump，并同步 digest_version/存档 Schema 策略）。
pub const CODEC_VERSION: &str = "CSIMCOD1";

/// canonical 编码器：小端、确定性、带 domain separation 头。
pub struct CanonicalWriter {
    buf: Vec<u8>,
}

impl CanonicalWriter {
    /// 开始一个带域分隔的编码流：CSIMCOD1 | module | kind | …字段…
    pub fn new(module: &str, kind: &str) -> Self {
        let mut w = Self { buf: Vec::new() };
        w.str(CODEC_VERSION);
        w.str(module);
        w.str(kind);
        w
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// usize 以 u64 编码（平台字长无关，2.9 口径）。
    pub fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }

    /// 变长字节：u64 长度前缀 + 原始字节。
    pub fn bytes(&mut self, b: &[u8]) {
        self.u64(b.len() as u64);
        self.buf.extend_from_slice(b);
    }

    /// 字符串：UTF-8 字节 + u64 长度前缀。
    pub fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }

    /// 序列前缀：u64 元素数 + 各元素由调用方逐个写入。
    pub fn seq_len(&mut self, len: usize) {
        self.usize(len);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 字节布局由规范直接推导（小端），断言精确字节。
    #[test]
    fn integer_layout_is_little_endian_fixed_width() {
        let mut w = CanonicalWriter::new("t", "x");
        w.u8(1);
        w.bool(true);
        w.u16(0x0203);
        w.u32(0x0405_0607);
        w.u64(0x0809_0a0b_0c0d_0e0f);
        w.i64(-1);
        assert_eq!(
            w.as_bytes()[w.as_bytes().len() - 24..],
            [
                1u8,
                1u8,
                0x03, 0x02,
                0x07, 0x06, 0x05, 0x04,
                0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ][..]
        );
    }

    /// 变长前缀与域分隔头。
    #[test]
    fn domain_header_and_varlen_prefix() {
        let mut w = CanonicalWriter::new("sim-agent", "AgentState");
        w.str("ab");
        let b = w.into_bytes();
        // 头：CODEC_VERSION | "sim-agent" | "AgentState"，各带 u64 长度前缀
        assert!(b.starts_with(&[8, 0, 0, 0, 0, 0, 0, 0]), "CODEC_VERSION 长度前缀=8");
        assert!(b.windows(8).any(|s| s == b"CSIMCOD1"));
        assert!(b.windows(9).any(|s| s == b"sim-agent"));
        assert!(b.windows(10).any(|s| s == b"AgentState"));
        // 尾部："ab" = 长度 2 + 'a' 'b'
        assert_eq!(&b[b.len() - 10..], &[2, 0, 0, 0, 0, 0, 0, 0, b'a', b'b']);
    }

    /// 同输入两次编码逐字节一致（确定性）。
    #[test]
    fn encoding_is_deterministic() {
        let f = || {
            let mut w = CanonicalWriter::new("m", "k");
            w.u64(42);
            w.str("稳");
            w.seq_len(2);
            w.i64(-7);
            w.i64(7);
            w.into_bytes()
        };
        assert_eq!(f(), f());
    }

    /// 浮点禁入由类型系统保证：编码器不提供 f32/f64 方法（本测试为契约记录）。
    #[test]
    fn no_float_entry_points() {
        let mut w = CanonicalWriter::new("m", "k");
        w.u32(0x3f80_0000); // 若需记录位模式，以整数显式写入
        // 头 34 字节（CSIMCOD1/m/k 各 8 字节长度前缀）+ u32 4 字节
        assert_eq!(w.as_bytes().len(), 38);
    }
}
