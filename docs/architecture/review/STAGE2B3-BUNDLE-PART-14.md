# STAGE2B3-BUNDLE PART 14（本卷文件：docs/architecture/rfc-agent-state-machine.md, docs/architecture/replay-schema.md, docs/architecture/rfc-lod-draft.md, docs/architecture/Stage-3-Task-List.md, docs/architecture/crate-authority-list.md, docs/benchmarks/Stage-2B-Exit-CHECKLIST.md, docs/benchmarks/Stage-3-Exit-CHECKLIST.md, docs/benchmarks/Stage2B-concurrent-stress.txt, docs/architecture/review/_manifest_stage2b3.txt）
<!-- commit 0bd384ab7874d9180273c7e0c87106b6e7d723e3；与 ZIP 同源；粘贴时请整卷一条消息发送 -->


===== FILE: docs/architecture/rfc-agent-state-machine.md =====

# RFC：Agent 状态机与受阻状态语义（Stage 3 T1，债务 #7）

- 状态：Proposed（2026-11-22；实装随本 RFC 一并落盘于 crates/sim-agent）
- 关联：EC-01（过期结果兜底）、ADR-NEW-17 条文 10（前向进展不变量+兜底三候选）、
  ADR-NEW-20（预算让步/请求队列）、EC-08（诊断分层）、2.3（AgentId 解耦）、
  decision-log 债务 #7

## 1. 问题

主文档多处引用"受阻状态"（EC-01 兜底"标记受阻"、ADR-NEW-17 条文 10"有限 tick 内
获得路径或进入明确定义的受阻状态"），但状态集、进入/退出条件、上限行为从未定义。
债务 #7 即此。

## 2. 状态集

| 状态 | 含义 | 终态 |
| --- | --- | --- |
| `Spawning` | 已分配 AgentId（2.3 高水位），等待进场 | 否 |
| `Active` | 正常活动（持有路径/移动中） | 否 |
| `Blocked { reason, since_tick, attempts }` | 受阻：前向进展不变量承诺的明确定义状态 | 否（必须有限转移） |
| `Despawned` | 已离场（终态；ID 不复用） | 是 |

受阻原因（`BlockedReason`）与触发源一一对应：

- `Unreachable`：寻路结果不可达（FindOutcome::Unreachable，ADR-NEW-17 条文 10 兜底候选
  "标记受阻"）；
- `RecomputeLimitExceeded`：过期结果重算超限（recompute_limit，EC-01 策略块）；
- `MarkBlockedFallback`：预算让步路径兜底 MarkBlocked（ADR-NEW-20/24 预算轴）。

## 3. 转移表（全集；未列出的转移一律非法并拒绝）

| # | 转移 | 条件 | 语义 |
| --- | --- | --- | --- |
| 1 | （无）→ Spawning | 分配器 alloc | 进场登记 |
| 2 | Spawning → Active | 进场 tick 到达 | 开始活动 |
| 3 | Active → Blocked | 任一 BlockedReason 触发 | attempts=1、since_tick=当前 tick |
| 4 | Blocked → Blocked | 再次受阻 | attempts+1，刷新 reason/since_tick |
| 5 | Blocked → Active | 重寻路成功 | 受阻解除（unblock） |
| 6 | Blocked → Despawned | 取消 / 超时退役 / attempts 超上限 | 受控终止（见 §4） |
| 7 | Active/Spawning → Despawned | 取消 / 到达目的地离场 | 正常离场 |

## 4. 受阻上限与前向进展

- `block_limit`（默认 3，与 recompute_limit、TX_RETRY_LIMIT 口径一致）：受阻累计
  attempts 超过上限时**自动退役**（→ Despawned），转移记录 `Retired`——这是
  "有限 tick 内获得路径或受阻、受阻亦有界"的收口，杜绝永久受阻悬挂。
- 前向进展不变量（ADR-NEW-17 条文 10）在本状态机的落点：任何 Agent 的 Blocked
  驻留必须以 #5/#6 结束，attempts 单调递增保证可终止性。

## 5. 确定性口径

- 转移记录 `StateTransition { agent_id, from, to, tick }` 按发生顺序追加（稳定序=
  调用序；调用方系统排序由 2.9 条 5 保证）；
- `digest()`：全部转移记录经 canonical 编码（sim-codec，module="sim-agent"、
  kind="AgentLifecycle"）→ BLAKE3（sim-digest）——两跑转移序列 digest 相等即状态机
  级可复现；
- 受阻转移配套诊断事件 `AGT-0001`（Diagnostic 层，EC-08；不改权威状态）。

## 6. 与既有语义的衔接

- EC-01 兜底三候选中"标记受阻"= 本状态机 #3（MarkBlockedFallback / Unreachable）；
  "局部可行路径/沿用旧路径有效段"不改变状态机（仍是 Active，只是路径降级）。
- ADR-NEW-20 让步路径：让步本身**不**触发受阻（请求仍在队列，Agent 保持 Active 等待）；
  只有让步路径的兜底 MarkBlocked 才进入 Blocked。
- AgentRegistry（Alive/Despawned）为旧粗粒度登记，与本状态机并存；集成期后合并
  （AgentState 为超集），合并前以 AgentLifecycle 为权威。

## 7. 开问题（随集成定案，不阻塞本 RFC）

- Blocked 期间的经济/LOD 交互（受阻 Agent 是否暂停计费）→ 随 T3/T4；
- 超时退役的"超时"计量单位（受阻 tick 数上限 vs attempts 上限）——当前 attempts
  上限=确定性工作量口径，时长上限待 2.10 口径复核后补充。


===== FILE: docs/architecture/replay-schema.md =====

# 回放输入 / 模拟配置 Schema（Stage 3 T2，债务 #10）

- 状态：Proposed（2026-11-22；实装随本 schema 一并落盘于 crates/sim-replay）
- 关联：2.7（机器性能调整"写入存档和回放输入"）、2.9 条 8（玩家命令带生效 tick 并
  记录供回放）、ADR-NEW-14 条 6（LOD 切换动作记录）、EC-03（Schema 版本化/迁移
  可诊断口径）、ADR-NEW-19（canonical 编码规则共用）

## 1. 定位

回放输入是**权威运行的输入侧完整记录**：种子+模拟配置+全部输入事件。回放等价=
同输入两次运行 digest 相等。与存档（EC-03）分工：存档记录**状态**（含在途任务），
回放输入记录**输入**；存档可在元数据中引用 ReplayInput digest 以绑定回放身份。

## 2. Schema（version = 1）

```
ReplayInput {
    schema_version: u16,        // =1；变更语义须 bump（EC-03 bump 规则同口径）
    seed: u64,                  // 全局种子（ADR-NEW-22 口径）
    tick_rate_num: u64,         // tick 频率分子（dt=den/num 派生，2.4 from_ratio）
    tick_rate_den: u64,
    events: [InputEvent],       // 记录序=发生序；回放按 tick 提交（见 §4）
}

InputEvent =                       // 判别 tag u8
  | Command { command_id: u64, effective_tick: u64, agent_id: u64,
              opcode: u32, arg0: i64, arg1: i64 }        // 2.9 条 8（对应 sim-core PlayerCommand）
  | ConfigChange { tick: u64, key: u64, value: i64 }      // 2.7 tick 边界配置变更（key=稳定键哈希）
  | LodSwitch { tick: u64, entity_id: u64, from_level: u8, to_level: u8 }  // ADR-NEW-14 条 6
  | BudgetAdjustment { tick: u64, system_id: u64, per_tick_cap: u64 }      // 2.7 预算调整
```

- 编码：canonical 小端（sim-codec，module="sim-replay" kind="ReplayInput"），
  与存档共用底层编码规则、顶层 Schema 独立（ADR-NEW-19 决策）；
- 尾字节拒绝、版本不符可诊断（RPL-0001）——EC-03 同口径。

## 3. 回放等价（验收判据）

录制运行（Recorder 收集 events）→ encode → decode → 回放运行（Replayer 按 tick
提交事件）→ 两运行 digest 相等。digest 覆盖：状态 + 输入事件序列。

## 4. 提交语义

- Command 按 `effective_tick` 生效（2.9 条 8；录制时即带生效 tick）；
- ConfigChange / BudgetAdjustment / LodSwitch 仅允许 tick 边界（录制 API 断言
  tick 单调非降；回放在各 tick 的事件按记录序提交=确定性）；
- 回放不重算决策：事件即事实（寻路/预算让步等系统级派生行为由确定性模拟自身重放）。

## 5. 版本 bump 规则

新增可选事件类型=可 bump 可不 bump（解码端拒绝未知 tag 时必须 bump 并提供迁移）；
字段语义/顺序/编码变更必须 bump；bunch 规则与 EC-03 Schema 版本一致（迁移失败可诊断）。


===== FILE: docs/architecture/rfc-lod-draft.md =====

# RFC：LOD（细节层级）草案

- 日期：2026-11-22
- 状态：Proposed（草案，Stage 3 T4；债务 #14"LOD 无 Stage 归属/命名/切换条件/更新频率"收口）
- 关联：主文档 2.7（三级 LOD+预算即代码）、ADR-NEW-14 条 6（LOD 切换确定性）、
  ADR-NEW-21（SimScalar）、replay-schema.md（LodSwitch 事件已定 schema v1）、债务 #14

## 1. 层级命名与语义（2.7"具体层级命名"收口）

| 层级 | 名称 | 更新频率 | 语义 |
| --- | --- | --- | --- |
| L0 | High（全精度） | 每 tick | 全部权威状态逐 tick 更新 |
| L1 | Medium（节流） | 每 2 tick 一次更新 | 隔 tick 更新（未更新 tick 保持上一状态） |
| L2 | Low（简化） | 每 4 tick 一次更新 | 简化运动模型（直线推进到中间路点） |

- 层级是**实体粒度**的质量分级（对单个 Agent/车辆生效），与 EC-04 降级
  （系统预算压力、L1/L2/L3、系统粒度）是**正交的两个轴**：两者都必须以确定性
  信号触发，但互不替代。
- 中/低层级改变权威状态（更新频率与简化模型），故其切换属确定性边界事项。

## 2. 切换条件（确定性信号，ADR-NEW-14 条 6）

1. **允许的触发信号**（全部确定性）：
   - 实体属性（如距玩家视点的网格距离——视点本身是确定性输入）；
   - 确定性工作量单位（如本 tick 预算消耗比例、实体计数）；
   - 玩家命令显式指定（CommandPayload::SetLod，记录于回放输入）。
2. **禁止的触发信号**：墙钟、CPU/内存占用、帧率、渲染负载、线程调度
   （违反 2.9 条 7 确定性边界）。
3. **切换动作必须记录**：`LodSwitch{tick, entity_id, from_level, to_level}`
   （replay-schema.md Schema v1 已定义）同时入存档（EC-03 元数据扩展项）与
   回放输入——与预算调整、LOD 记录同口径（2.7"写入存档和回放输入"）。

## 3. 简化模型约束（L1/L2 正确性）

1. 简化运动模型不得改变**路径合法性**：中途路点必须落在原路径有效段上
   （与 ADR-NEW-17 条文 9"只保证合法性"同口径）。
2. 降级/LOD 切换不得破坏强读档等价：切换状态与切换计划（下次更新 tick）须
   随存档恢复（EC-03 预算与降级类字段同组扩展）。
3. L1 隔 tick 更新的 tick 间隙内，实体状态视为"保持"（显式语义，非未初始化）。

## 4. 验证计划（实施时）

- 同种子两跑（含 LOD 切换）digest 一致；
- 切换动作序列入回放→回放 digest 与录制一致（sim-replay LodSwitch 事件闭环）；
- 禁止信号守护：clippy disallowed-methods 已覆盖墙钟入口（EC-12），
  切换函数签名不接受非确定性参数（类型层防）。

## 5. 不在本草案范围

- LOD 具体阈值数值（随 2.10 参数基线体系与实测定案，标记待定）；
- 视点/相机系统实现（属渲染/工具层）；
- LOD 与降级联动策略（L2/L3 降级期是否强制全实体降 LOD——留待实测与 ADR）。


===== FILE: docs/architecture/Stage-3-Task-List.md =====

# Stage 3 任务清单：模拟语义扩展（重建版）

- 状态：执行中（2026-11-22 起；重建以替代缺失的原始路线图"后续阶段"部分——债务 #19 同款问题，
  主文档 L1038-1040 注）
- 依据：主文档 2.2 分层（经济层）、2.7 LOD、2.9 条 8 回放输入、EC-01 兜底、EC-03（WorldId 字段）、
  EC-04（L2/L3 暂停经济）、EC-10 多世界边界（L770-790）、ADR-NEW-14 条 6（LOD 切换确定性）、
  ADR-NEW-18/21（经济数值独立 i64）、ADR-NEW-19（摘要纳入权威状态含经济）；
  债务 #7/#10/#14（decision-log）
- 范围：Agent 状态机、回放输入/模拟配置 schema、经济系统基础、LOD 草案、多世界边界预留
- 不在范围：E 议题三件（在途行程时间/有向边独立容量/准入规则）——独立立项，不挂本清单；
  UI/渲染（非权威侧）；MFD→降级（债务 #31 转 C 终局，永久纯统计）
- 执行纪律：每完成一个任务 git 提交推送 + decision-log《Stage 3 执行日志》记录；
  状态用语按 0.2 词表；条件核销清单对应项随任务勾销。

## 前置（S2，先行独立小步）

| # | 任务 | 内容与依据 | 交付物 | 验收标准 | 依赖 |
| --- | --- | --- | --- | --- | --- |
| T0 | 债务 #28 tx_id 重试实装 | ADR-NEW-24 决策 4/债务 #28 定案："沿用原 tx_id 按全序重排入队重试"；重试上限 3，超限转受阻状态+诊断事件 | crates/sim-network 重试队列（TxRetryQueue）+错误码登记 | 重试沿用原 ID 测试；上限 3 转受阻+诊断测试；与 TxLimiter（BudgetExceeded）衔接 | 无 |

## 任务分解

| # | 任务 | 内容与依据 | 交付物 | 验收标准 | 依赖 |
| --- | --- | --- | --- | --- | --- |
| T1 | Agent 状态机（债务 #7） | 状态集与转移条件**含受阻状态语义**（进入=寻路 Unreachable/重算超限 MarkBlocked/受阻上限；退出=重寻路成功/取消/超时）；与 EC-01 兜底（ADR-NEW-17 条文 10）、ADR-NEW-20 让步路径衔接；转移事件确定性（入摘要+诊断事件） | Agent 状态机 RFC 文档（docs/architecture/）+ sim-agent 状态机实装 | 状态转移表全覆盖测试；受阻进入/退出测试；两跑转移序列 digest 一致 | T0 |
| T2 | 回放输入/模拟配置 schema（债务 #10） | 2.7/2.9 条 8 承诺"写入存档和回放输入"：命令序列（生效 tick）、配置变更（tick 边界）、LOD 切换记录、预算调整记录；canonical 编码+Schema 版本化（EC-03 口径）；回放=种子+配置+命令日志→同 digest | sim-save 回放模块（或 sim-replay）+schema 文档+录制/回放 API | 回放等价测试（录制运行回放→digest 相等）；schema 版本拒绝测试 | T1 |
| T3 | 经济系统基础（sim-economy） | 经济数值独立 i64（ADR-NEW-18/21：10^12 上限、不用 SimScalar）；确定性 RNG 派生流（ADR-NEW-22 "economy" namespace）；EC-04 L2/L3 暂停经济语义；摘要纳入（ADR-NEW-19 模块清单） | sim-economy 最小确定性循环（预算/收支/事件） | 两跑同种子经济序列 digest 一致；i64 上界溢出防护测试；暂停/恢复语义测试 | T1 |
| T4 | LOD 草案（债务 #14） | 三级命名/切换条件/更新频率**定案草案**（ADR 文档，Proposed→评审）；实装锚点=ADR-NEW-14 条 6（切换条件确定性信号+动作入存档回放）；不做完整 LOD 引擎，先契约 | docs/architecture/adr/ LOD 草案 + sim-agent/sim-traffic 切换钩子接口 | 草案含切换条件确定性证明路径；钩子接口编译通过+切换记录结构测试 | T1 |
| T5 | 多世界边界（EC-10） | SimulationContext+WorldId 类型边界预留（不要求第二套 ECS，L788）；WorldId 入存档/摘要（EC-03 字段）；两实例状态归属隔离 | sim-core 世界上下文类型 + 隔离测试 | 两实例并行运行互不污染（各 digest 独立可复现）；WorldId 入档入摘要测试 | T1 |

## 退出条件（提案，验收时按条件核销清单核销）

1. Agent 状态机受阻语义完整可测：进入/退出/上限转移全覆盖，转移序列确定性可复现。
2. 回放等价：种子+配置+命令日志 → 回放 digest == 原运行 digest（债务 #10 闭合）。
3. 经济系统两跑 digest 一致，数值契约（i64 上限）有防护测试。
4. LOD 草案 ADR 落盘（Proposed），切换确定性条款有实装锚点。
5. 多世界边界隔离测试通过，WorldId 入档入摘要。
6. 条件核销清单对应项（债务 #7/#10/#14 相关）勾销。

## 执行顺序

T0 → T1 → {T2, T3} 并行 → {T4, T5} 收口。每个任务=独立提交
（格式：`feat(stage3-Tn)` 或 `docs(stage3-Tn)`）。


===== FILE: docs/architecture/crate-authority-list.md =====

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
| crates/sim-replay | 回放输入/模拟配置（2.9 条 8、债务 #10） | ✅ | 严格 | canonical 编码+Schema 版本化 |
| crates/sim-world | 多世界上下文（EC-10 WorldId/SimulationContext 预留） | ✅ | 严格 | 世界身份入摘要域 |
| crates/sim-test-support | 测试基建（EC-07） | ❌ | 无（不受限） | fixtures/harness/property/benchmark |
| crates/dep-verify | 依赖验证件（Step 8） | ❌ | 无（不受限） | 五件套验证 |
| spikes/*（4 个） | Spike 验证件 | ✅（验证口径） | 严格 | 保留至正式 crate 迁移完成；不得被 crates/ 依赖 |

注：sim-error/codec/digest/rng/simscalar/sim-save 为 Stage 1 各任务（T2/T5/T6/T7/T8）交付，
先登记分类，建库时须同步补 clippy.toml。权威侧禁令（disallowed）：HashMap/HashSet、
f32/f64、Instant::now/SystemTime::now——详见 clippy-layering-REPORT.md。

## 依赖方向约束（dep_graph_check.py 强制）

1. sim-core 仅可依赖基础设施 crate（sim-codec、sim-error）；不得依赖业务层/工具层（模拟核心是业务依赖根）。（2026-11-19 Stage 2C C3a 修订：预算注册表需 canonical 编码与错误分层；两基础设施 crate 均为叶子 crate，无环。）
2. 权威 crate 不得依赖非权威 crate（sim-view、sim-test-support、dep-verify）。
3. 任何 crate 不得依赖 sim-view（工具层不可被模拟依赖；2.2"工具层不能成为模拟状态的唯一管理者"）。
4. crates/ 与权威层不得依赖 spikes/（验证件不得成为正式依赖）。
5. sim-test-support 仅作为 dev-dependency 被引用（测试基建不入运行时）。


===== FILE: docs/benchmarks/Stage-2B-Exit-CHECKLIST.md =====

# Stage 2B 退出条件核对（2026-11-19）

范围：主文档"### Stage 2B：并行分片与确定性验证"（L980-1003）四条退出条件 + 第二轮评审（ADR-NEW-24）硬门禁。
环境：Windows x86_64、rustc 1.100.0（R4 跨平台未验）。

## 退出条件（主文档四条）

| # | 退出条件 | 判定 | 证据 |
| --- | --- | --- | --- |
| 1 | 执行顺序确定性得到验证 | ✅ | sim-parallel `commit_order_is_stable_key_order`、`submission_order_independence`；场景级 `submission_order_independence_scenario`（crates/sim-parallel/tests/thread_independence.rs） |
| 2 | 线程数无关性测试通过（1、2、8 线程摘要一致） | ✅ | `thread_count_independence_1_2_8`；`concurrent_queue_thread_and_chaos_independence`；压测 1/2/8+混沌 digest 全一致（docs/benchmarks/Stage2B-concurrent-stress.txt） |
| 3 | 任务提交顺序有明确契约 | ✅ | 契约=稳定键 (phase_id, task_key) 全序 + 主线程串行提交（sim-parallel `run_parallel`；与 ADR-NEW-15 P1-02 全序键口径一致） |
| 4 | 并行与单线程结果能够按照设计规则对比 | ✅ | `parallel_matches_serial_reference`（同纯函数同排序，仅执行并发度不同）+ `chaos_injection_does_not_change_result` |

## 第二轮评审硬门禁（ADR-NEW-24 条件）

| # | 门禁 | 判定 | 证据 |
| --- | --- | --- | --- |
| 1 | 让步路径强读档等价专项 | ✅ | sim-save `yield_state_strong_readback`（降级期让步计数>0 的存档点强读档等价成立） |
| 2 | 并发队列压测 | ✅ | crates/sim-pathfinding/tests/concurrent_queue_stress.rs（QUOTA=200 强制让步路径）+ examples/concurrent_stress.rs（200×200/500 请求/churn）；会计恒等式=终态事件数==请求数+重算数 |
| 3 | R3 线程无关（1、2、8 线程 + 注入延迟） | ✅ | chaos_spin 确定性忙等注入（禁 sleep/墙钟，符合 2.9 条 7）；1/2/8 + chaos42 四组摘要一致 |
| — | R4 跨平台摘要一致性 | ⛔ 移交 | 不属 Stage 2B 范围；保留为 R4 发布前门禁（债务台账既有条目） |

## 压测关键数据（release）

- 场景：200×200 双向网格（40k 节点 / 79.6k 边）、500 请求、churn 每 8 tick 删 4 边（单向删口径）、Q=65536/tick
- 结果：ticks=22、completed=502（=500 请求 + 2 笔过期重算再终态）、unreachable=0、yield_events=857、stale_recomputes=2、stale sweeps=9643、pending_left=0
- 墙钟：1 线程 341.5ms / 2 线程 283.5ms / 8 线程 236.3ms（并行扫描段加速 1.45×；串行权威段占比大，符合 Amdahl 预期）
- 摘要：`ca23d3951a5ced0a3658fc78bdb29c983e3c1f06777b62e6004d8d34254fc28c`，1/2/8 线程与混沌注入四组完全一致

## 限定与遗留

- 单平台（Win x86_64）单工具链（1.100.0）实测；R4 跨平台保留为发布门禁。
- 规则网格幸存者偏差（GEMINI 风险条目）：迷宫/非均匀代价/reopen 触发场景随债务 #30 细则补遗测试补齐。
- 债务 #27（机制 B→A 切换阈值）数值仍待定，本压测 dup_rate 与耗时数据为其定案输入之一。
- Q=65536 **已定案**（2026-11-19 决策3 投票 1a，9:1；2026-11-20 X5 单一来源 `sim-pathfinding::TICK_QUOTA` 钉死，本行原"占位待回填"口径作废）。

## 结论

Stage 2B 退出条件 **4/4 达成**；硬门禁 Stage 2B 范围内 **3/3 达成**（R4 移交）。**Stage 2B 关闭**（2026-11-19）。


===== FILE: docs/benchmarks/Stage-3-Exit-CHECKLIST.md =====

# Stage 3 退出条件核对（2026-11-22，S4）

依据：docs/architecture/Stage-3-Task-List.md（退出条件，2026-11-22 重建清单）。
核对口径：证据指向提交/测试/文档，未达成项如实标注。

## 退出条件逐条核对

| # | 条件 | 结果 | 证据 |
| --- | --- | --- | --- |
| 1 | T0 债务 #28 重试队列实装（沿用原 tx_id、上限 3、超限受阻+诊断） | ✅ | crates/sim-network TxRetryQueue + 3 测试（16/16）；NET-0001 登记；提交 55a439f |
| 2 | T1 Agent 状态机：状态集+转移表+受阻语义+确定性 | ✅ | docs/architecture/rfc-agent-state-machine.md + sim-agent AgentLifecycle（8/8）；AGT-0001；提交 67ad3c2 |
| 3 | T2 回放输入/模拟配置 schema 定义且可回放验证 | ✅ | docs/architecture/replay-schema.md + crates/sim-replay（5/5，录制 digest==回放 digest）；RPL-0001；提交 5e193bb |
| 4 | T3 经济基础：独立整数上限、确定性派生流、摘要纳入、暂停语义 | ✅ | crates/sim-economy（4/4：两跑 digest 相等、10^12 溢出/下溢防护、EC-04 L2/L3 暂停）；提交 b8df1af |
| 5 | T4 LOD 草案：命名/切换条件/更新频率+确定性信号+记录入档入回放 | ✅ | docs/architecture/rfc-lod-draft.md（阈值数值=待定，属草案范围外声明）；提交 bc69b2e |
| 6 | T5 多世界最小预留（EC-10 WorldId/上下文） | ✅ | crates/sim-world（3/3：世界摘要隔离、高水位拒绝回退）；提交 876ced7 |

**结论：6/6 达成，Stage 3 清单任务全部完成。**

## 限定与移交

1. T1/T2/T3/T5 为**基础原语**，与模拟主循环的集成（run_tick 统一驱动）属后续阶段；
   AgentRegistry 旧粗粒度与 AgentLifecycle 并存待集成期合并（rfc-agent-state-machine.md §5）。
2. T4 阈值数值（各层距离/频率）待定，随实测与 2.10 体系定案；
   LOD 与降级联动策略未定（草案 §5）。
3. 遗留债务移交后续：#20 预算滚动状态生命周期、#23 联动组 G4、#27 阈值定案
   （审计保持候选）、R4 跨平台（发布门禁）、E 议题三件（独立立项）。
4. 全部新 crate 已进 CI 双矩阵与依赖方向检查（规则 1-5 PASS）。


===== FILE: docs/benchmarks/Stage2B-concurrent-stress.txt =====

   Compiling sim-parallel v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-parallel)
   Compiling sim-rng v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-rng)
   Compiling sim-test-support v0.0.1 (D:\AIWORK\CITYSIM\crates\sim-test-support)
   Compiling sim-pathfinding v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-pathfinding)
    Finished `release` profile [optimized] target(s) in 1.07s
     Running `target\release\examples\concurrent_stress.exe`
== Stage 2B 并发队列压测（200x200, 500 请求, churn 每 8 tick 删 4 边, Q=65536/tick）==
口径：budget_yield_count=累计让步事件；stale_recomputes=路网轴重算（两轴分离）；recompute 分母=stale 查询总数
threads=1 chaos=0: digest=ca23d3951a5ced0a ticks=22 completed=502 unreachable=0 yields=857 recomputes=2 tx_ok=2 tx_rej=0 sweeps=9643 pending_left=0 wall=341.5ms
  full digest: ca23d3951a5ced0a3658fc78bdb29c983e3c1f06777b62e6004d8d34254fc28c
threads=2 chaos=0: digest=ca23d3951a5ced0a ticks=22 completed=502 unreachable=0 yields=857 recomputes=2 tx_ok=2 tx_rej=0 sweeps=9643 pending_left=0 wall=283.5ms
  full digest: ca23d3951a5ced0a3658fc78bdb29c983e3c1f06777b62e6004d8d34254fc28c
threads=8 chaos=0: digest=ca23d3951a5ced0a ticks=22 completed=502 unreachable=0 yields=857 recomputes=2 tx_ok=2 tx_rej=0 sweeps=9643 pending_left=0 wall=236.3ms
  full digest: ca23d3951a5ced0a3658fc78bdb29c983e3c1f06777b62e6004d8d34254fc28c
threads=4 chaos=42: digest=ca23d3951a5ced0a ticks=22 completed=502 unreachable=0 yields=857 recomputes=2 tx_ok=2 tx_rej=0 sweeps=9643 pending_left=0 wall=243.4ms
  full digest: ca23d3951a5ced0a3658fc78bdb29c983e3c1f06777b62e6004d8d34254fc28c
线程无关（1/2/8）+混沌无关（4）摘要一致：PASS


===== FILE: docs/architecture/review/_manifest_stage2b3.txt =====

MANIFEST — CitySim-rs STAGE2B3 AUDIT package (v2)
基准 commit: 0bd384ab7874d9180273c7e0c87106b6e7d723e3（2026-11-22）
v2 修订：补 v1 遗漏 9 文件（globaldigest-golden-vectors.txt、sim-test-support 六模块
文件、sim-core/src/lib.rs）；v1 的 sim-traffic/lib.rs 与 mfd_prime.rs"截断"系传输
（聊天粘贴超长被切尾）非包内缺失——v2 分卷包按文件边界切卷防截断。
口径说明：
1. 审计范围=从未外部审计的工程面（Stage 2B、Stage 3 T0-T5、②′ 修测原语）。
2. sim-pathfinding/src/lib.rs 内含 TickBudget/quota_yield_count（A 项）。
3. criterion_aligned_loss/point_resolution_ok/steady_window 在 sim-traffic/src/lib.rs
   尾部（C 项）；mfd_prime.rs 为 ②′ fixture。
4. 文本与仓库该 commit 逐字节同源；CRLF 差异不计。
5. 测试基线：workspace 49 组 test result: ok、0 失败（S4 终验）。
