# 模拟主循环集成退出记录（I1–I4）

- 日期：2026-11-23
- 状态：退出核对（对应 docs/architecture/integration-design.md 验收判据；Stage 3 集成收口）
- 测试基线：sim-engine 9/9（cargo test -p sim-engine）、sim-replay 7/7、clippy 零警告

## integration-design.md 验收 4 判据对照

| # | 判据 | 结果 | 证据测试 |
| --- | --- | --- | --- |
| 1 | 同种子两跑 GlobalDigest（逐 tick 摘要+chain）相等 | ✅ | two_run_determinism、r2_smoke_scenario_end_to_end（前断言） |
| 2 | 强读档等价：save（snapshot）→restore→继续跑 digest 一致 | ✅ | strong_readback_equivalence_via_snapshot、r2_smoke_scenario_end_to_end（后断言）、snapshot_roundtrip_is_lossless（往返无损回归） |
| 3 | 回放等价：录制输入→重放 digest 一致 | ✅ | replay_equivalence_via_recorded_commands |
| 4 | R2 缩小场景端到端（Fixture::smoke：100 Agent、命令/事务/交通注入、min(30) tick） | ✅ | r2_smoke_scenario_end_to_end |

## 集成发现的三个真 bug（均修复并回归）

1. **命令重复生效**：due_commands 为纯查询（≤tick 全量返回）无应用游标→逐 tick 重复
   生效。修复=applied_command_ids 游标（BTreeSet 去重）。
2. **GlobalDigest 同键覆盖**：逐条 record("PathOutcome"/"TxApplied") 被同键覆盖只剩最后
   一条。修复=每 tick 新建摘要+事件缓冲聚合单条记录（CommandAppliedSet/PathOutcomeSet/
   TxAppliedSet）+滚动 chain。
3. **Recorder 静默丢命令**：record 按 effective_tick 单调校验，命令提交序生效 tick 可
   倒序→OutOfOrderTick 被丢弃→重放少命令 ID 重排。修复=Command 事件按提交序记录豁免
   单调（全序由 (effective_tick, command_id) 回放期重排），单调约束仅 tick 边界事件
   （replay-schema.md 已同步语义）。

## 教训

- 测试期望先算清语义（r2_smoke 首版比不同 tick 数的链——第三次应验）。
- 快照往返用 Debug 全等对比定位字段级缺失最快。
- Recorder 语义与命令提交序的关系必须写进 schema（契约先行缺一角的代价）。

## 限定与移交

- R2 正式门禁（10k×1k）仍由 CI r2-deterministic 占位（启用前置=GlobalDigest 正式化+
  fixture 规模）；本次为缩小场景端到端验证。
- 集成层未含 LOD 切换/降级触发到 run_tick 的实接（RFC/预算通道冻结中，随对应裁决）。
- 全量回归：workspace 测试全绿（提交前验证）。
