# crate 权威/非权威清单（债务 #15 落点，2.2 架构分层定案）

- 日期：2026-11-19（Stage 1 T1）
- 依据：主文档 2.2 架构分层、2.9 确定性边界（权威/非权威状态划分）、EC-12 契约即代码
  （clippy 分层已实测，docs/benchmarks/clippy-layering-REPORT.md）
- 约束执行：本清单 + `.github/scripts/dep_graph_check.py`（CI 门禁）+ 各权威 crate 包根
  `clippy.toml`（disallowed-types/methods）。**新增 crate 须先在本表登记**。

## 分层与权威分类

| crate | 2.2 架构层 | 权威 | clippy.toml | 说明 |
| --- | --- | --- | --- | --- |
| crates/sim-core | 模拟核心 | ✅ | 严格 | Tick 推进与模拟状态管理；不得依赖任何 UI/工具层 |
| crates/sim-agent | Agent 系统 | ✅ | 严格 | Agent 身份、状态与行为 |
| crates/sim-network | 路网系统 | ✅ | 严格 | 道路拓扑、路网修改与数据 |
| crates/sim-pathfinding | 寻路系统 | ✅ | 严格 | 路径搜索与任务调度 |
| crates/sim-traffic | 交通系统 | ✅ | 严格 | 交通流量及相关模型 |
| crates/sim-economy | 经济及城市系统 | ✅ | 严格 | 城市模拟能力 |
| crates/sim-view | 可视化与工具层 | ❌ | 无（不受限） | 展示状态、诊断、调试；不得成为模拟状态唯一管理者 |
| crates/sim-error | 错误分层（EC-08） | ✅（基础） | 严格 | Fatal/Recoverable/Diagnostic + SimErrorContext |
| crates/sim-codec | canonical 编码器（ADR-NEW-19） | ✅ | 严格 | 存档/摘要共用底层编码规则 |
| crates/sim-digest | GlobalDigest（ADR-NEW-19） | ✅ | 严格 | 权威状态摘要（R2 oracle） |
| crates/sim-rng | RNG 协议（ADR-NEW-22） | ✅ | 严格 | 派生流 + Stream64 |
| crates/simscalar | SimScalar（ADR-NEW-21） | ✅ | 严格 | Q32.32 定点 |
| crates/sim-save | 存档/读档（EC-03） | ✅ | 严格 | 保存并恢复语义 |
| crates/sim-parallel | 确定性并行执行（2.9 条 6） | ✅ | 严格 | 快照只读+稳定键排序+串行提交 |
| crates/sim-test-support | 测试基建（EC-07） | ❌ | 无（不受限） | fixtures/harness/property/benchmark |
| crates/dep-verify | 依赖验证件（Step 8） | ❌ | 无（不受限） | 五件套验证 |
| spikes/*（4 个） | Spike 验证件 | ✅（验证口径） | 严格 | 保留至正式 crate 迁移完成；不得被 crates/ 依赖 |

注：sim-error/codec/digest/rng/simscalar/sim-save 为 Stage 1 各任务（T2/T5/T6/T7/T8）交付，
先登记分类，建库时须同步补 clippy.toml。权威侧禁令（disallowed）：HashMap/HashSet、
f32/f64、Instant::now/SystemTime::now——详见 clippy-layering-REPORT.md。

## 依赖方向约束（dep_graph_check.py 强制）

1. sim-core 不得有任何依赖（模拟核心是依赖根）。
2. 权威 crate 不得依赖非权威 crate（sim-view、sim-test-support、dep-verify）。
3. 任何 crate 不得依赖 sim-view（工具层不可被模拟依赖；2.2"工具层不能成为模拟状态的唯一管理者"）。
4. crates/ 与权威层不得依赖 spikes/（验证件不得成为正式依赖）。
5. sim-test-support 仅作为 dev-dependency 被引用（测试基建不入运行时）。
