# ADR-NEW-26：需求压力降级信号通道（MFD→BudgetRegistry）

- 编号：ADR-NEW-26（暂定，入仓重编号时保留映射）
- 状态：Proposed（2026-11-19 草案，待评审）
- 日期：2026-11-19
- 关联：ADR-NEW-25 决策 2/3c（MFD 权威性边界与耦合验证）、ADR-NEW-14 C2 节（降级触发）、
  ADR-NEW-20（ID/在途任务）、债务 #31（MFD 模型形式缺陷）
- 来源：决策2 投票（10/10 有条件采纳）条件"3c 生产代码路径 + 专用耦合线程无关测试"。

## 背景

决策2 投票要求 MFD→降级/背压信号必须有生产代码路径并通过专用耦合线程无关验证
（通用并行证据不替代耦合验证）。同时 V1（3a 独立参照）已实测出 MFD 模型形式缺陷
（债务 #31：点队列 q(k) 凹形、平台型/三角型线性自由流分支假设不成立），
"债务 #31 裁决前 MFD 仅统计展示、不作降级触发依据"是既有处置。两者存在张力：
既要生产路径，又不能让有缺陷的模型影响权威降级。

## 问题

MFD（非权威派生统计）如何安全地接入权威降级状态机（BudgetRegistry），既满足
3c 的"生产代码路径"要求，又不违反"债务 #31 裁决前 MFD 不作降级触发依据"的限制？

## 决策

1. **需求压力通道**：sim-traffic 新增确定性定点输出
   `demand_pressure_bp(k_ref_permille) -> u32`（0..=10000）：
   `pressure = min(10000, (active/edge_count × 1000) × 10000 / k_ref)`，
   **k_ref==0 返回 0**（2026-11-19 决策4 修订：fail-safe 改向——分母未知时宁可不触发
   降级，不得默认最大压力；原返回 10000 方向反了，MiniMax 指出）。
   k_ref（特征密度）=q 首次 ≥90%·q_max 的密度（版本化定义；由辨识模型导出并随
   拟合参数入档入回放配置，旧口径 k_c=argmax q=采样上沿随密度阶梯漂移 2.5 倍，
   8 家评审一致弃用）。特征密度语义变更时强读档等价边界=同版本内（MiMo）。
   输出进入 `BudgetRegistry::note_demand_pressure(pressure_bp)`。
2. **有效压力 = max(工作量, 需求)**：`effective_bp = max(used_bp, demand_bp)`
   驱动降级触发/恢复门（连续 trigger_ticks / recover_ticks、闭开区间语义沿用
   ADR-NEW-14 C2 节 S2 定案口径）。取 max 不加权——加权聚合已被决策3 投票
   （千问Go!/MiniMax：稀释掩盖）否决。
3. **需求单独触发封顶 L1**：demand_only_trigger 时 Normal→L1 且**不得**推 L2/L3；
   工作量通道不受此限（可推 L2/L3）。理由=债务 #31 未裁决前限制 MFD 影响面：
   需求压力可触发"轻降级"（新请求延迟入队），但重降级（暂停经济/仅维护）只信
   工作量单位（ADR-NEW-14 条文）。
4. **状态入档入摘要**：demand_bp、demand_streak 写入 encode_state（G3 口径），
   强读档等价覆盖需求通道状态。
5. **恢复=双通道同时回落**：effective_bp < recover_bp 连续 recover_ticks。
6. **耦合验证**：专用测试 crates/sim-traffic/tests/mfd_coupling.rs——
   车辆快照→sim-parallel 并行聚合（含 chaos_spin 忙等）→demand_pressure_bp→
   note_demand_pressure→end_tick→GlobalDigest，1/2/8 线程 + chaos 0/42 +
   纯串行参考三路摘要逐一相等。

## 考虑过的替代方案

| 方案 | 结论 | 理由 |
| --- | --- | --- |
| MFD 直接推 L2/L3 | 否决 | 债务 #31 未裁决；模型缺陷会污染重降级 |
| 仅展示不入权威（无生产路径） | 否决 | 决策2 投票条件明确要求生产代码路径 |
| 加权混合压力（cap 加权） | 否决 | 决策3 投票教训：空闲系统稀释饱和系统 |
| 独立"需求降级"状态机 | 否决 | 降级等级是唯一权威状态（千问Go!/MiniMax 口径），第二状态机引入分裂 |

## 影响

- ADR-NEW-25 3c 条件达成一半（生产代码路径 + 专用耦合测试已实装）；
  余"MFD 信号触发降级的实际收益验证"随债务 #31 模型形式裁决后补。
- 2026-11-19 决策4 已裁决（债务 #31=凹形扩展启动）：分母口径已修订为 k_ref
  特征密度（见决策 2 修订注），新 k_ref 定案前需求通道维持 L1 封顶（千问Go!）。
- ADR-NEW-14 C2 节 L1/L2/L3 映射补"需求通道封顶 L1"约束（已随 S2/V2 落盘）。

## 验证方法

- sim-core：demand_pressure_triggers_l1_only_with_hysteresis、
  demand_alone_cannot_reach_l2_l3、demand_state_is_covered_by_encode。
- sim-traffic：mfd_degradation_coupling_is_thread_independent（1/2/8）、
  …_is_chaos_independent（0/42）、…_matches_serial_reference。
- 全部通过即 3c 生产路径与耦合验证达成（2026-11-19 实测）。

## 当前状态

Proposed（草案）——生产代码路径与测试已实装并通过；正式裁决随下一轮评审。
**通道冻结（2026-11-20；2026-11-21 更新）**：原 Set D FAIL（148/142‰）已被债务闭合
审计证实为 q>k 测量缺陷误差地板（原证据无效）；整改后 Set E 确认轮（预注册 fixture
9a73ee0）2/3 拓扑 PASS、32×32 自由流 133‰ FAIL——**未全过确认**。债务 #31 处置待
负责人裁决（追认转 C / MiniMax E 议题修基后再战）。回滚≠解禁，本通道维持冻结：
MFD 不作降级/背压触发依据，在 #31 处置定案并重新验证前**不得接入权威降级路径**；
k_ref 特征密度口径保留为待定案（随重开评估一并定案）。
