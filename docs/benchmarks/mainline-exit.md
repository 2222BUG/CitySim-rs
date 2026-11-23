# 主干实接退出记录（W1–W4）

- 日期：2026-11-23
- 状态：退出核对（对应 integration-design.md"2026-11-23 主干实接（W1–W3）"增量节）
- 测试基线：sim-engine 14/14、sim-core 20/20、sim-agent 9/9、sim-economy 4/4、sim-traffic 23+1 ignored+3、workspace 全绿、clippy 零警告

## W1–W3 交付对照

| 步骤 | 交付 | 验证证据 |
| --- | --- | --- |
| W1 降级实接 | run_tick 按 BudgetRegistry.level() 分支（L1 新请求延迟/L2 配额封顶 DEGRADATION_L2_QUOTA_CAP+经济自动暂停/恢复/L3 仅维护）；三类输入事件实接（BudgetAdjustment→set_per_tick_cap、ConfigChange→入摘要、LodSwitch→lod_levels 入摘要入档）；submit_input_event 即录制（回放等价四类全覆盖） | 四类事件确定性、回放等价含四类、降级分支确定性（含 EC-04 暂停映射轨迹：L3→pause→resume）3 测试 |
| W2 LOD 实接 | GridTraffic::step_with_lod 移动更新按 2^lod 分频（tick 低位对齐确定性信号；注入/出网照常）；run_tick 步骤 4 接 lod_levels | lod_switch_gates_traffic_deterministically（同种子一致+非恒等） |
| W3 经济联动 | income/upkeep=AgentLifecycle::active_count×费率 确定性派生（费率入 EngineState 快照）；EC-04 暂停映射经 W1 | economy_follows_active_agents_deterministically（两跑一致/空载零收支/despawn 变化非恒等） |
| W4 收口 | integration-design.md 增量节+本退出记录+decision-log | 提交序列 8d3d24d→ea473ad→47652a0→本次 |

## 教训（本次新增）

- 测试期望先算清语义第 4 次应验：降级轨迹=运行中曾达 L2+ 且曾暂停，非终态 paused
  （恢复后自动 resume 是正确行为）。
- EngineConfig/spawn_agent 等 API 签名猜写 5 连错——写测试前 grep 实文（第 5 次
  重申）。
- decision-log old_string 未命中第 5 次（W1-W4 节当时尚不存在）——插入新节应先确认锚点节存在。

## 限定与移交

- DEGRADATION_L2_QUOTA_CAP=TICK_QUOTA/4 为候选值（随 ADR-NEW-14 阈值回填）。
- LOD 分频现作用于交通移动更新；Agent/寻路侧 LOD（rfc-lod-draft 全量范围）随模拟内容扩展。
- W3 费率语义=每 agent 费率×活跃数（EngineConfig.income_per_tick 字段语义已演进为
  per-agent，文档口径以本记录为准）。
