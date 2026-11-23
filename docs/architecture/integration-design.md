# 模拟主循环集成设计（sim-engine）

- 状态：Proposed（实现随 I1-I4 四步推进，每步提交）
- 日期：2026-11-23
- 关联：主文档 2.8（并行计算与串行提交）、2.9（确定性边界）、EC-03（存档最小集）、
  EC-11（读档等价）、replay-schema.md、Stage-3-Task-List（T0-T5 原语层交付）
- 背景：Stage 3 交付了确定性原语（时间/预算/身份/寻路/路网/交通/经济/回放/多世界），
  但各自独立、没有统一的 tick 驱动。本设计把原语接成一个**确定性主循环**，
  并以两跑 digest / 强读档等价 / 回放等价三条端到端判据验收。

## 1. tick 处理顺序契约（固定，不得随实现漂移）

`run_tick()`（单 tick = 原子步，tick 边界=唯一存档点）按下列**固定顺序**执行：

1. **命令生效**：CommandLog::due_commands(tick) 取本 tick 生效命令（全序
   (effective_tick, command_id)），逐条应用到权威状态；命令事件记入回放 Recorder。
2. **寻路处理**：对寻路请求队列按请求 ID 稳定序逐个 `PathFinder::request_with_budget`
   （TickBudget 每 tick 重置为 Q=65536 单一来源）；Complete/Yielded/Unreachable
   三态记账（budget_yield_count / quota_yield_count 分离——审计3 口径）。
3. **路网事务**：TxRetryQueue.due() 重试批次先行（按 tx_id 全序）→ 新事务
   TxLimiter::admit → Network::publish（原子）；被拒事务 offer 进重试队列
   （上限 TX_RETRY_LIMIT=3，超限受阻+NET-0001 诊断）。
4. **交通步进**：GridTraffic::step（确定性 id 序）。
5. **经济结算**：EconomyLedger::settle_tick（EC-04 L2/L3 暂停语义）。
6. **降级评估**：BudgetRegistry::end_tick（确定性工作量单位，ADR-NEW-14 口径；
   需求通道默认冻结=ADR-NEW-26）。
7. **摘要折叠**：本 tick 全部权威状态变化按 §3 清单折入 GlobalDigest。
8. **时钟推进**：SimClock::advance()（u64 tick，唯一时间入口）。

顺序不变量：同一 tick 内任何步骤不得读系统时钟、不得依赖 HashMap/HashSet
迭代顺序（2.9 条 3/4）；步骤 1-6 内的并行只读按 2.9 条 6（快照+稳定键串行提交）。

## 2. 组件接线表

| 步骤 | 组件 | 状态出口（新增=引擎可见） |
| --- | --- | --- |
| 1 | sim-core CommandLog/CommandIdAllocator | id_high_water（已有） |
| 2 | sim-pathfinding PathFinder/TickBudget | 计数器（已有 pub） |
| 3 | sim-network Network/TxLimiter/TxRetryQueue | version、tx_log、counts/blocked（新增 state_export） |
| 4 | sim-traffic GridTraffic | fold_digest（已有）、last_step_population |
| 5 | sim-economy EconomyLedger | digest（已有） |
| 6 | sim-core BudgetRegistry | encode_state（已有） |
| 身份 | sim-agent AgentLifecycle/AgentIdAllocator | block_attempts、高水位（新增 state_export） |
| 世界 | sim-world SimulationContext | world_id、seed（已有） |

## 3. GlobalDigest 纳入清单（防 fold_digest 漏字段教训，逐 tick 折叠）

clock.tick；命令事件（id/生效 tick/参数）；寻路结果（request_id/结果类/展开数）；
路网 version+事务应用序列（tx_id 序）；交通 fold_digest（含 last_step_population、
goal/next_id/pending——既有修复口径）；经济 digest；预算 encode_state（含
rolling/level/需求冻结态）；AgentLifecycle digest（含 block_attempts）；
各分配器高水位；world_id/seed。**任何权威状态字段变更必须同步本清单**。

## 4. 存档与回放接线

- 存档（EC-03 最小集）：引擎 `save()`=canonical 编码 §3 清单对应全状态
  （含在途请求队列、重试队列、让步计数、回放游标），仅 tick 边界；`load()` 读档后
  一致性检查+摘要比对（SAV-0001/SAV-0002 语义）。
- 回放（replay-schema v1）：步骤 1 的命令事件与步骤 3 的 ConfigChange 类事件
  记入 Recorder；回放=清空命令来源、由 Replayer.due(tick) 驱动步骤 1，其余
  步骤同构→回放 digest 必须等于录制 digest。
- 强读档等价：run N→save→load→run 至 M 的 digest == 直接 run 至 M（R2 强形式，
  "保存并恢复"语义——ADR-NEW-16 定案）。

## 5. 验收判据（I2-I4）

1. 两跑一致：同种子两跑逐 tick digest 相等；不同种子区分（digest 不等）。
2. 强读档等价（上式，接 sim-save 口径）。
3. 回放等价：录制-回放 digest 相等（接 sim-replay 口径）。
4. R2 缩小场景（fixture：1k Agent×200 tick 或 Fixture::smoke）端到端跑通。

## 6. 非目标

并行执行（2.9 条 6 的 run_parallel 接入留待 Stage 2B 集成级）；LOD 切换实装
（rfc-lod-draft 阈值未定）；MFD→降级（#31 转 C 终局）。
