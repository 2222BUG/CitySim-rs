# AI 评审任务书：Stage 2C 两项决策投票（阈值确认 + ADR-NEW-25 裁决）

- 日期：2026-11-19
- 评审对象：CitySim-rs（Rust + Bevy 开源城市模拟；确定性核心承诺：同种子逐 tick 可复现、强读档等价）
- 回传渠道：GitHub issue（见文末）；或交由转发人转交
- 预计用时：10~15 分钟（代码级审查者 20 分钟）

## 署名要求（不署名不计票）

报告署名须给出以下之一：真实名字 / GitHub 昵称 / AI 模型具体型号（如 `DeepSeek-V4.1flash`、`qwen4-max`、`MiniMax-M3.1-Flash`；只写"某 AI"等泛称视同未署名）。

## 背景（30 秒）

Stage 2C（MFD、动态路网与背压）实施面退出 6/6（证据：docs/benchmarks/Stage-2C-Exit-CHECKLIST.md）。
现在有两项决策需要外部投票：
1. **决策 1：预算/降级阈值确认**（数值拍板，ADR-NEW-14 C2 细化节）
2. **决策 2：ADR-NEW-25（MFD 交通模型与验证方案）裁决**（四选一）

## 决策 1：预算与降级阈值（附实测依据，请逐行表态）

| # | 参数 | 候选值 | 依据 | 表态 |
| --- | --- | --- | --- | --- |
| 1a | 寻路全局每 tick 节点展开配额 Q | 65536 | 实测 worst=398 展开/请求（40k 节点规则网格，G5-stress）；机制 B 单请求上界 ≤2\|V\|；65536 留 100× 余量，Q 的意义=极端场景保险丝 | 接受 / 调整为 ___ |
| 1b | 降级触发比例 L1/L2/L3 | 70% / 90% / 100% | 纯工程惯例值（无实测支撑，唯一硬约束=只依赖工作量单位、确定性） | 接受 / 调整为 ___ |
| 1c | 降级恢复比例（迟滞） | 50% | 防抖：恢复须低于最低触发档；迟滞区间 50~70% | 接受 / 调整为 ___ |
| 1d | 路网事务每 tick 上限 | 1024 笔 | 实测常态 1 tx/tick、churn 压测 4 边/8 tick；1024=极端余量 | 接受 / 调整为 ___ |

约束（不可谈）：阈值触发只依赖确定性工作量单位（ADR-NEW-14 已裁决 Accepted）；
数值可谈，但改动须给出理由；确认后回填 ADR-NEW-14 并转定案。

## 决策 2：ADR-NEW-25 裁决（四选一）

裁决口径：**采纳 / 有条件采纳 / 拒绝 / 延后**。

ADR-NEW-25 要点：
- 模型形式：三角基本图（候选），参数（v_free/k_c/q_max/w）必须由 T-Bench-1 微观基准定点校准产生，禁止拍脑袋。
- MFD 权威性边界：默认只作非权威统计与降级/背压输入，不反馈权威路径选择（反馈需另行 ADR）。
- 三项独立验证（2.6 要求）：适用范围（误差阈值 10%/20%/25% 为占位）、参数校准（确定性重校准逐位一致）、耦合方式（线程无关摘要不变）。
- T-Bench-1：40×40 双向网格、密度阶梯持续注入、200 tick 稳态采样、双向扫描（滞回检测）、逐 tick 折叠 GlobalDigest。
- 确定性判据：同种子两跑逐点 (k,q) 一致；注入顺序无关性。

已有证据（可复核）：
- mfd-tbench1.txt：密度 2→256 实测 q 8→249‰edges，呈基本图形态，三角拟合定点输出。
- sim-traffic 4 测试全绿（含同种子两跑一致、拟合确定性）。
- **注意**：3a 的误差阈值（10%/20%/25%）与三角形式均为占位候选，校准后回填。

## 代码级审查（每轮至少一家，请核对源码）

重点文件（ZIP 内含全文）：
- `crates/sim-traffic/src/lib.rs`——确定性声明（BTreeMap 全序、event_u64 派生 OD、无浮点/时钟）是否成立？
  容量排队口径（每目标格每 tick 放行 cap 辆、id 序先到先行）是否会产生不公平饥饿？
- `crates/sim-core/src/budget.rs`——降级状态机的迟滞逻辑是否正确？边界（used=恰 7000bp）归属是否清晰？
- `crates/sim-traffic/examples/tbench1.rs`——采样口径（末 tick 瞬时 q）是否足以代表稳态流量？（如认为应取窗口均值，请在裁决中指出）

## 裁决格式（请原样返回，两项各自一段）

```
## 决策 1 投票
- 署名：<名字 / GitHub 昵称 / AI 模型具体型号>
- 1a/1b/1c/1d：接受 或 调整为 ___（附一句理由）

## 决策 2 裁决：ADR-NEW-25
- 署名：<同上，具体型号>
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：（1-3 句）
- 验证条件 / 修改要求：（逐条）
- 对代码级审查重点的意见（如有）：
- 新增风险（如有）：
```

## 约定

- 只依据链接/ZIP 材料原文判断，材料未覆盖的点标"未见证据"。
- **拒绝和延后是合法裁决**，按证据直说。
- 回传：贴到投票 issue（见分享消息），或回复给转发人。

## 链接三件套

- 仓库：https://github.com/2222BUG/CitySim-rs
- 材料包 ZIP：https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/CitySim-rs-DECISION-2C.zip
- 全量仓库 ZIP：https://github.com/2222BUG/CitySim-rs/archive/refs/heads/main.zip
- 关键 RAW：ADR-NEW-25 https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/adr/ADR-NEW-25-MFD%E4%BA%A4%E9%80%9A%E6%A8%A1%E5%9E%8B%E4%B8%8E%E9%AA%8C%E8%AF%81%E6%96%B9%E6%A1%88.md
- ADR-NEW-14 https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/adr/ADR-NEW-14-%E9%A2%84%E7%AE%97%E4%B8%8E%E9%99%8D%E7%BA%A7%E5%BF%85%E9%A1%BB%E7%A1%AE%E5%AE%9A%E6%80%A7.md
- 退出核对 https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/Stage-2C-Exit-CHECKLIST.md
- T-Bench-1 数据 https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/mfd-tbench1.txt
