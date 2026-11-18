# Stage 1 任务清单：核心模拟基础（重建版）

- 状态：执行中（2026-11-19 起；重建以替代缺失的"原有 Stage 1 任务清单"，债务 #19）
- 依据：主文档 2.2 架构分层、2.3 AgentId、2.4 SimScalar、2.9 确定性边界、3.x EC-01~EC-05、
  4.x EC-08/EC-12、EC-11 测试基建；裁决=A​​DR-NEW-15/16/19/20/21/22（decision-log《2026-11-19 正式裁决记录》）
- 范围：模拟核心、Agent 身份、基础状态管理、确定性边界落实（主文档 Stage 1 段 L950-954）
- 不在范围：路网/寻路（Stage 2A）、并行分片（Stage 2B）、LOD/预算降级（Stage 2C）、UI（可视化层）
- 执行纪律：每完成一个任务 git 提交推送 + decision-log"Stage 1 执行日志"记录；条件核销清单
  （decision-log）对应项随任务勾销；"条件未清不得开工对应模块"（0.2 注记规范）。

## 任务分解

| # | 任务 | 内容与依据 | 交付物 | 验收标准 | 依赖 |
| --- | --- | --- | --- | --- | --- |
| T1 | workspace/crate 划分骨架 | 按 2.2 分层（模拟核心/Agent/路网/寻路/交通/经济/可视化/工具）划 crate，权威/非权威分类（债务 #15）；权威 crate 根置严格 clippy.toml（EC-12 已实测） | crates/ 目录结构、crate 间依赖约束（模拟核心不依赖 UI 等）、权威清单 | 依赖方向测试（如 cargo-deny 或分层编译验证）；clippy 分层不误伤非权威 | 无（首任务） |
| T2 | SimScalar 正式 crate | ADR-NEW-21：Q32.32(i64)、舍入"就近 0.5 远离零"、乘除 i128 中间值、from_ratio；迁移 spikes/simscalar-spike 的 12 测试+黄金值为正式测试 | crates/simscalar/（+黄金测试入 docs/benchmarks/ 记录） | 12 测试迁移全过；与 spike 黄金值一致（dt_raw=143,165,577 等） | T1 |
| T3 | 时间与命令日志 | 2.9 条 7/8：u64 tick 权威时间（禁系统时钟/秒累积）；玩家命令带生效 tick、tick 边界落定、记录供回放输入 | crates/sim-core 时间模块+命令日志结构 | 两跑同种子命令序列 digest 一致；命令生效 tick 测试 | T1 |
| T4 | Agent 身份 | 2.3：AgentId 分配器（u64 高水位）与 Entity 解耦；分配器状态入存档（EC-03） | crates/agent 身份模块 | 高水位防复用测试（save-restore-spike 已验证语义，迁移正式测试） | T1 |
| T5 | RNG 协议实装 | ADR-NEW-22：双模式（纯派生 derive_seed/event_u64 + Stream64 顺序流 16B）、SplitMix64 v1、RNG_PROTOCOL_VERSION；namespace 注册表+注册期 64 位碰撞校验（债务 #25，条件核销项） | crates/rng 协议 crate + namespace 注册表 | spikes/rng-spike 9 测试迁移全过；碰撞校验测试（构造碰撞被拒）；协议版本钉死测试 | T1 |
| T6 | canonical 编码器 + GlobalDigest | ADR-NEW-19（最优先，债务 #11/#20/#23 上游）：BLAKE3-256、canonical 小端编码（domain separation 模块+类型标识）、digest_version u16、纳入全部权威状态（含预算队列/让步状态，NEW-19 条件③）；黄金测试向量入 docs/benchmarks/ | crates/sim-codec（编码器）+ crates/sim-digest（GlobalDigest） | 黄金向量测试（digest_version=1）；同状态两构建 digest 稳定；编码规则=存档共用底层（顶层 Schema 独立） | T1 |
| T7 | 存档/读档 EC-03 实装 | ADR-NEW-16 七类最小集+三类 ID 高水位+让步状态；save 仅 tick 边界；"保存并恢复"；读档一致性检查+GlobalDigest 比对；未知 Schema 可诊断拒绝；强读档等价测试入 R2（条件核销项） | crates/sim-save（正式实现） | save-restore-spike 9 测试迁移全过+强读档等价（N 存档读档跑至 M == 直接跑至 M）；篡改拒绝 | T2-T6 |
| T8 | 错误分层骨架 | EC-08：Fatal/Recoverable/Diagnostic + SimErrorContext（entity_id 语义=AgentId，与 2.3 解耦口径一致） | crates/sim-error | 分层枚举+上下文结构+诊断事件通道；错误码规范文档片段 | T1 |
| T9 | 确定性契约测试接入 | EC-07/EC-11：sim-test-support 对接（run_scenario harness）、2.9 八条契约逐条测试或 lint 证据（禁浮点/禁 HashMap 迭代依赖/稳定排序/显式顺序/u64 时间/命令 tick）、R2 场景骨架（缩小场景先跑通，10k×1k 全量随核心成熟接入） | tests/ 集成 + R2 场景 fixture | 2.9 八条逐条证据表；同种子两跑 digest 相等；线程无关性留 R3（裁决 #9） | T2-T8 |

## 退出条件（提案，验收时按条件核销清单核销）

1. 核心模拟循环可运行且可复现：同种子两次独立运行任意 tick 数 GlobalDigest 相等。
2. 强读档等价：N tick 存档读档跑至 M == 直接跑至 M（R2 强形式，A​​DR-NEW-16 定案语义）。
3. T2/T5/T6 正式 crate 测试全过，且与 Spike 黄金值一致（Spike 证据链延续）。
4. 2.9 八条契约逐条有测试或 lint 证据（对应 EC-12 契约即代码）。
5. GlobalDigest 黄金向量入仓（digest_version=1），R2 oracle 可用。
6. 条件核销清单中 Stage 1 对应项（NEW-15 落地验证、NEW-16 读档等价、NEW-19 编码器/黄金/清单、NEW-25 碰撞校验等）勾销。

## 执行顺序

T1 → {T2, T5, T6, T8} 并行 → {T3, T4} → T7 → T9 收口（贯穿集成测试）。
每个任务=独立提交（格式：`docs(step1-Tn)` 或 `feat(step1-Tn)`）。
