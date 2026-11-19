# CitySim-rs 债务闭合审计任务书（#27/#28/#29/#30/#31）

- 版本：2026-11-20
- 发起：项目负责人（维护者代发）
- 审计对象：最近闭合/定案的一批决策债务（执行=维护者 AI 一方完成，缺外部审计）
- 回传渠道：GitHub issue（见群发消息）；**报告者署名：推荐具体 AI 型号/版本（如 DeepSeek-V4.1flash、qwen4-max、MiniMax-M3.1-Flash、GPT-6），泛称同样计票，完全未署名不计票**
- 材料包：CitySim-rs-DEBTS-AUDIT.zip（含源码+证据文件+本任务书+MANIFEST）

## 一、为什么需要这次审计

这批债务的"实现—测试—判定"全部由维护者 AI 一方完成（无独立验证方）。历史记录显示
外部审查命中率高：ADR-NEW-24 轮 QWEN4-MAX 抓出 2 条真代码缺陷、决策2 轮
MiniMax-M3.1-Flash 抓出 15 条（含 v_free 31 倍失真）。本次审计的使命=**在正式入账前
找缺陷**，包括推翻判定（如"转 C 不成立"是合法审计结论）。

## 二、审计范围（6 项）

### A. 债务 #31 执行面与"转 C"终局（最重）
背景：决策4（10 票 A7:B2:C1）裁决凹形扩展启动+预注册终止条件"Set D 或第二拓扑
任一 FAIL → 转 C，不再延后"。执行=W1 拟合器+W2 Set D 复测+W3 落盘。
实测：40×40 自由流 148‰ FAIL / 64×64 142‰ FAIL（判据 100‰），转折/饱和 PASS。
请核对：
- A1 Set D fixture 是否真"预注册"（种子 0x00C0_FFEE_2222_0000+i 不相交、拟合 2→256/
  留出 3→384 交错、8 种子、双拓扑、判据常量=sim-traffic 单一来源）——是否存在
  "看数据后调 fixture"的空间？
- A2 终止条件执行是否忠实（两拓扑自由流 FAIL→转 C）；"转 C"是否应再经投票确认？
- A3 fit_rational_saturation_detailed 的"自由估计+越界披露+物理可行解回退"口径
  是否符合决策4"越界判失败"（豆包）原意？回退到可行解是否算"把约束当估计"？
- A4 自由估计斜率 1242/1271‰ 超物理界（>1 车/边/tick）——判定为 MiniMax E 议题
  （基准无交通物理语义）实证是否成立？还是测量/单位口径 bug？
- A5 稳态诊断（40×40 1/8、64×64 0/8 档稳态）仅披露不阻断——是否削弱 FAIL 判定的
  独立性（不稳态数据上的误差判据有效吗）？

### B. 债务 #30 A* 细则补遗（5 测试）
crates/sim-pathfinding/tests/astar_edge_cases.rs：reopen_recovers_optimal…/
equal_cost_paths_keep_first_discovered_predecessor/shuffled_edge_insertion…/
csr_offsets_survive_beyond_u16_edge_count/h_consistency_invariant…。
请核对是否真覆盖决策2/3 要求（reopen 差分、等代价前驱稳定、乱序注入、CSR u32、
h 一致性不变量），测试构造是否有 gaming（如 reopen 图是否真触发 reopen）。

### C. 债务 #27 阈值候选（未定案，审推导）
docs/benchmarks/G5-threshold-baseline.txt 三场景（open/obstacle/maze）实测后，
候选判据="单次尝试展开>4096 或让步≥8 切机制 A"（dup_rate 被证伪，max 0.664≈解析
上界 2/3）。请核对：三场景代表性、4096/8 两阈值的推导是否过拟合场景、置信度。

### D. 债务 #28/#29 定案质量
#28=事务重试沿用原 tx_id 按全序重排（新 ID 需映射状态入档被否决）；#29=TickBudget
{remaining=Q}，Q=65536 占位、B_k≤min(b·2^k,L,remaining)。请核对与 ADR-NEW-14 条 3/
ADR-NEW-20/23 的一致性、Q 占位的风险。

### E. 近期代码质量（代码级审查重点）
- sim-traffic/src/lib.rs：fit_rational_saturation_detailed（联合网格辨识、平局取最小）、
  q_rational_permille（定点 +−×÷、就近舍入）、demand_pressure_bp（k_ref 口径）
- sim-traffic/examples/mfd_setd.rs：证据纪律（Tee 输出、稳态诊断披露）
- sim-core/src/budget.rs：需求通道（effective=max、L1 封顶、迟滞门、encode_state）
- sim-pathfinding/src/lib.rs：parent 记录松弛实际边、saturating 运算（决策4 修复项回归）
请核对确定性红线（2.9/ADR-NEW-21：无平台浮点、无 HashMap 迭代依赖、无时钟、
不静默环绕）与 clippy 权威侧口径。

### F. 文档一致性
"转 C"在 ADR-NEW-25（终局注）、ADR-NEW-26（通道冻结注）、decision-log（债务 #31
行+W1–W3 执行日志+变更历史）、条件核销清单四处口径是否一致；CI #[ignore] 测试
注释与转 C 一致；README/主文档是否需要状态同步。

## 三、裁决格式（每项 A–F 填一行；总体一行）

```
## 债务闭合审计记录
- 报告者：（署名——推荐具体 AI 型号/版本）
- A（#31 终局）：通过 / 有条件通过 / 不通过  —— 理由+修改要求
- B（#30）：…    C（#27）：…    D（#28/#29）：…
- E（代码级）：…（请附行号级证据）
- F（文档一致性）：…
- 总体结论：这批债务闭合可入账 / 有条件入账 / 需重做
- 新增缺陷（逐条）：
```

## 四、材料清单（ZIP 内）

- 本任务书；MANIFEST（基准提交+口径说明）
- crates/sim-pathfinding/{src/lib.rs, tests/astar_edge_cases.rs, examples/g5_threshold_baseline.rs}
- crates/sim-traffic/{src/lib.rs, examples/mfd_setd.rs, examples/mfd3a.rs, tests/mfd_coupling.rs}
- crates/sim-core/src/budget.rs
- docs/benchmarks/{mfd-3a-setd.txt, mfd-3a-reference.txt, G5-threshold-baseline.txt}
- docs/architecture/adr/{ADR-NEW-24, 25, 26}.md
- docs/architecture/decisions/decision-log.md

## 五、约定

- 只依据材料与仓库原文判断；未覆盖的点标"未见证据"
- **拒绝/不通过/要求重做是合法结论**，请按证据直说
- 代码级审查请至少一家附行号级证据（REVIEW-CONVENTIONS 第 3 条）
