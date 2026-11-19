# Stage 2C 退出核对（2026-11-19）

主文档 Stage 2C 退出条件（L1022-1034）逐条核对。证据均为本仓库实测（docs/benchmarks/ 与测试）。

## 退出条件逐条核对

| # | 退出条件 | 结论 | 证据 |
| --- | --- | --- | --- |
| 1 | 动态路网更新正确 | ✅ | sim-network 9 测试（topology_tx_is_atomic、cost_tx_updates_in_place、csr_adjacency_order_is_deterministic）；Stage-2A-Exit-CHECKLIST 5/5 |
| 2 | 事务发布没有中间态暴露 | ✅ | topology_tx_is_atomic（失败零改动+摘要不变）；concurrent_queue_stress 串行提交（Stage 2B）；sim-parallel commit 串行化 |
| 3 | 过期寻路结果按 EC-01 策略处理，重算有上限、无活锁 | ✅ | stale_check_two_phase/stale_aggregation_is_stable；G5-churn 重算率 0.0027（分母=请求数口径 0.015）、recompute_limit=3、concurrent_queue_stress pending_left=0 |
| 4 | 寻路任务具有明确预算和背压行为 | ✅ | TickBudget（ADR-NEW-20/29 实装）：global_budget_caps_total_expansions_per_tick、yield_then_fresh_budget_completes；机制 B 预算递增（ADR-NEW-24 决策 3）；并发压测 yields=857 让步有界 |
| 5 | 预算与降级触发只依赖工作量单位；注入延迟或改变线程数不改变摘要 | ✅ | sim-core budget：degradation_levels_follow_workload_only（触发/恢复=消耗比例，无墙钟）；线程无关：sim-parallel thread_count_independence_1_2_8 + chaos_injection_does_not_change_result + Stage2B-concurrent-stress 四组摘要一致（digest ca23d395…） |
| 6 | 交通模型具备可复现验证场景 | ✅ | T-Bench-1（ADR-NEW-25 决策 4）：sim-traffic 4 测试（tbench_same_seed_two_runs_agree 等）+ docs/benchmarks/mfd-tbench1.txt（40×40 密度阶梯 2~256，q 8→249‰ 呈基本图形态，三角拟合定点输出） |

## 限定与待办

- **阈值已定案**（2026-11-19 决策3 投票 1a/1b/1c/1d，原"候选待负责人确认"作废）：Q=65536、降级 70/90/100、恢复 50%、事务 1024。
- **MFD 阈值为占位候选**（ADR-NEW-25 3a 判据 10%/20%/25%）：校准后回填方可转 Accepted。
- ADR-NEW-25 与 ADR-NEW-14 C2 细化节均为 Proposed，待第 7 章评审流程裁决。
- 单平台单工具链实测（Windows x86_64 / rustc 1.100.0）；R3/R4 门禁口径沿 Stage 2B 结论。
- 债务 #27（机制 B→A 切换阈值）仍开放，不阻塞 2C。

## 结论

实施面退出 **6/6**。Stage 2C 实施收口；正式关闭仍待：①ADR-NEW-25 评审裁决、②ADR-NEW-14 C2 阈值确认。
