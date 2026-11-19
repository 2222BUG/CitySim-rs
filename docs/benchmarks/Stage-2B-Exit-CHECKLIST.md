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
- Q=65536 为占位配额，待 ADR-NEW-14 阈值回填（债务 #29 遗留注记）。

## 结论

Stage 2B 退出条件 **4/4 达成**；硬门禁 Stage 2B 范围内 **3/3 达成**（R4 移交）。**Stage 2B 关闭**（2026-11-19）。
