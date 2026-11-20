# AI 评审任务书：决策 5——债务 #31 复判（转 C 回滚与凹形扩展收口）

- 日期：2026-11-20
- 回传渠道：issue #26（https://github.com/2222BUG/CitySim-rs/issues/26）或交回委托人
- 基准提交：fb9fe47（Set D 证据 commit：ce4a705）
- 署名：**推荐**具体 AI 型号/版本（如 DeepSeek-V4.1、qwen4-max）或名字/GitHub 昵称；泛称同样计票，完全未署名不计票

## 一、背景（30 秒）

1. 2026-11-19 决策4（10 票 A7:B2:C1）裁决"凹形扩展启动"，附条件⑧："自由估计零密度斜率越界（>1000‰）判失败，禁止把物理约束当估计用"。
2. W1/W2 实装后 Set D 复测（V1 时代口径）报自由流 148/142‰ FAIL → W3 按预注册终止条件"转 C"（MFD 永久纯统计）。
3. 债务闭合审计（7 家，审计1.MD）确认转 C 程序执行忠实，但**证据基础四项 P0**：
   (a) k/q 采样口径不对称（flow 含到达车、active 不含）→ q>k 误差地板；
   (b) 判定边界自指（k_turn=fit.k_half，同拟合同数据 40×40 改绝对带全 PASS）；
   (c) "越界判失败"实现为"披露+回退可行解继续判定"，与条件⑧相反；
   (d) 证据文件与源码不同源（构建日志含已删除 import）。豆包复算：|7−6|/7=142.86‰ 恰为 64×64 FAIL 值。
4. 整改 X1–X6（c8a7a28/31019d2/ce4a705/125d71d/ef21a8e/fb9fe47）修复全部 P0，并补 p-norm p=2 广义族并行辨识（防"单族以偏概全"）。
5. Set D 复测（证据 commit ce4a705，证据文件自带 SETD_COMMIT）**判定翻转**。

## 二、复测数据（判定翻转）

| 族 | 40×40 自由流/转折/饱和 | 64×64 | 自由估计斜率 | 判定 |
|---|---|---|---|---|
| rational（有理饱和） | 83/118/56‰ 全 PASS | 80/112/89‰ 全 PASS | 1095/1092‰ >1000‰ | FAIL（binding，条件⑧淘汰） |
| **pnorm-p2** | **83/189/12‰ 全 PASS** | **48/120/20‰ 全 PASS** | 932/944‰ 物理可行 | **PASS** |

- 预选（拟合集 rel-SSE，锁定于留出评估前）：rational 27996 < pnorm 44390（40×40）→选 rational→Set D outcome=FAIL；64×64 同理。**若按"物理可行域内预选"，两拓扑均选 pnorm-p2→PASS**——这正是本次待裁决的预选规则问题。
- 稳态诊断：持续注入高密度档不稳态（0-1/8 档，back-half 敏感性 PASS 方向一致），已披露。
- 修复后原"自由流 FAIL"消失（q>k 误差地板消除），与豆包复算一致。

## 三、待裁决三项（请逐项表态）

| # | 事项 | 选项 |
|---|---|---|
| Q1 | **转 C 回滚确认**：撤销"2026-11-20 Set D FAIL→转 C（MFD 永久纯统计）"终局，债务 #31 改"凹形扩展成立"路线 | 确认回滚 / 维持转 C / 延后 |
| Q2 | **pnorm-p2 族写入 ADR-NEW-25 决策 1**（q=q_max·k/sqrt(k²+k0²)，定点 isqrt 实现）作为凹形扩展默认族（rational 保留为备选） | 采纳 / 有条件采纳 / 拒绝 |
| Q3 | **族预选规则**：候选族辨识后按"物理可行域内拟合集 rel-SSE 最小"预选（free estimate 越界族不得参与预选，binding 仅诊断） | 采纳 / 有条件采纳 / 拒绝+替代方案 |

## 四、代码级审查重点（本轮至少一家请逐行核对）

1. `fit_pnorm_concave_detailed`（crates/sim-traffic/src/lib.rs）：可行域约束（k0≥q_max）、free/feasible 双解、SSE 为相对误差平方和（与判据同构）。
2. `mfd_setd.rs` 族预选：`[family pre-selection] fit-set rel-SSE` 是否真的锁定于留出评估之前（防 gaming）。
3. 条件⑧口径：binding=true 是否已从判定输入中剔除（回退解仅诊断）。
4. X2 预算修复（crates/sim-pathfinding/src/lib.rs）：Unreachable 携带 expanded 扣减、quota_yield_count 与 budget_yield_count 分离。
5. 证据文件 mfd-3a-setd.txt 的 SETD_COMMIT=ce4a705 与源码版本一致性。

## 五、裁决格式（原样返回）

```
## 决策 5 裁决记录（债务 #31 复判）
- 署名：<AI 具体型号 / 名字 / GitHub 昵称>
- Q1 转 C 回滚：<确认回滚 / 维持转 C / 延后>——理由（1-2 句）
- Q2 pnorm-p2 族：<采纳 / 有条件采纳 / 拒绝>——理由 + 验证条件
- Q3 族预选规则：<采纳 / 有条件采纳 / 拒绝>——理由 + 替代方案（如拒）
- 整体意见：
- 新增风险（如有）：
- 代码级审查结果（如做）：
```

## 六、约定

只依据材料包与仓库原文判断，材料未覆盖的点标"未见证据"；**拒绝与延后是合法裁决**。
三项全通过 → #31 收口（凹形扩展成立+MFD→降级解禁进入重验证流程）；任一不过 → 按该项否决路径处置。

## 七、材料清单

任务书、ADR-NEW-25（含 3a 复测结果节）、ADR-NEW-26、mfd-3a-setd.txt（翻转证据）、
mfd-3a-reference.txt（V1 对照）、sim-traffic lib.rs（拟合器）、mfd_setd.rs（Set D 基准）、
sim-pathfinding lib.rs（预算修复）、sim-core budget.rs、decision-log.md（#31 全史+审计裁决记录）、MANIFEST.txt。
