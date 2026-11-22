# Stage 3 退出条件核对（2026-11-22，S4）

依据：docs/architecture/Stage-3-Task-List.md（退出条件，2026-11-22 重建清单）。
核对口径：证据指向提交/测试/文档，未达成项如实标注。

## 退出条件逐条核对

| # | 条件 | 结果 | 证据 |
| --- | --- | --- | --- |
| 1 | T0 债务 #28 重试队列实装（沿用原 tx_id、上限 3、超限受阻+诊断） | ✅（审计3 修复后） | crates/sim-network TxRetryQueue + 3 测试（16/16）；NET-0001 登记；提交 55a439f。审计3 发现计数随 due() 清零致上限不可达——已修复（counts 独立生命周期+运营路径测试，7015fed） |
| 2 | T1 Agent 状态机：状态集+转移表+受阻语义+确定性 | ✅（审计3 修复后） | docs/architecture/rfc-agent-state-machine.md + sim-agent AgentLifecycle（8/8）；AGT-0001；提交 67ad3c2。审计3 发现 attempts 跨 unblock 重置（block→unblock→block 永不退役）——已修复（累计单调+运营路径测试） |
| 3 | T2 回放输入/模拟配置 schema 定义且可回放验证 | ✅（原语级，限定） | docs/architecture/replay-schema.md + crates/sim-replay（5/5，录制 digest==回放 digest）；RPL-0001；提交 5e193bb。限定：回放等价验证使用玩具累加器非权威循环（审计3），接真实模拟循环的端到端验证属集成阶段 |
| 4 | T3 经济基础：独立整数上限、确定性派生流、摘要纳入、暂停语义 | ✅ | crates/sim-economy（4/4：两跑 digest 相等、10^12 溢出/下溢防护、EC-04 L2/L3 暂停）；提交 b8df1af |
| 5 | T4 LOD 草案：命名/切换条件/更新频率+确定性信号+记录入档入回放 | ✅ | docs/architecture/rfc-lod-draft.md（阈值数值=待定，属草案范围外声明）；提交 bc69b2e |
| 6 | T5 多世界最小预留（EC-10 WorldId/上下文） | ✅ | crates/sim-world（3/3：世界摘要隔离、高水位拒绝回退）；提交 876ced7 |

**结论：6/6 原语级交付达成（集成级未闭合——审计3 F 口径修正，2026-11-22），
Stage 3 清单任务全部完成。**

## 限定与移交

1. T1/T2/T3/T5 为**基础原语**，与模拟主循环的集成（run_tick 统一驱动）属后续阶段；
   AgentRegistry 旧粗粒度与 AgentLifecycle 并存待集成期合并（rfc-agent-state-machine.md §5）。
2. T4 阈值数值（各层距离/频率）待定，随实测与 2.10 体系定案；
   LOD 与降级联动策略未定（草案 §5）。
3. 遗留债务移交后续：#20 预算滚动状态生命周期、#23 联动组 G4、#27 阈值定案
   （审计保持候选）、R4 跨平台（发布门禁）、E 议题三件（独立立项）。
4. 全部新 crate 已进 CI 双矩阵与依赖方向检查（规则 1-5 PASS）。
