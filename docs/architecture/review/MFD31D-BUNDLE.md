# CitySim-rs-MFD31D 单文本材料包（与 ZIP 同基准 1d8b779）



---

## FILE: AI-REVIEW-BRIEF-MFD31D.md

# 决策6 投票任务书：债务 #31 处置（Set E 确认轮之后）

- 轮次：决策6（#31 终局处置）
- 回传渠道：issue #26（https://github.com/2222BUG/CitySim-rs/issues/26）
- 基准提交：1d8b779（材料包内证据均带 commit 标注）
- 署名：推荐具体 AI 型号/版本或 GitHub 昵称（泛称亦计票；完全未署名不计票）
- 约定：只依据链接/材料包原文判断，材料未覆盖的点标"未见证据"；**拒绝和"不选任一项"是合法裁决**

## 一、背景（30 秒）

CitySim-rs 的 MFD 交通模型验证（债务 #31）经历三次终局反复：

1. Set D 复测 FAIL →"转 C"（MFD 永久纯统计）；
2. 债务闭合审计（7 家）证实原证据无效（q>k 测量缺陷=误差地板、判定边界自指）→转 C 撤销；
3. 整改后 Set E 确认轮（预注册 fixture 9a73ee0）**2/3 拓扑 PASS、32×32 自由流 133‰ FAIL（判据 100‰）**，
   back-half 同值 133（稳定信号非瞬态）。

现在需要对 #31 做终局处置。**回滚≠解禁**：无论选哪项，MFD→降级通道继续冻结。

## 二、二选一（请明确表态）

| 项 | 内容 | 后果 |
|---|---|---|
| **① 追认转 C** | 按"任一 FAIL"预注册精神，MFD 永久仅作非权威统计展示，不作降级/背压触发依据 | #31 关闭为终局；pnorm-p2 族扩展随文档归档；MFD 用途收窄不可逆 |
| **② MiniMax E 议题修基再战** | 先修微观基准的交通物理语义（候选三件：车辆在途行程时间、有向边独立容量、准入/发车规则），扩族（或复测 pnorm-p2）后开新确认轮再定 | #31 保持开放；工程量约 1-2 轮（基准改造+复测+投票）；若修基后仍 FAIL 则追认 ① |

## 三、重点问题（请逐项给意见）

- Q1 32×32 free=133‰（n=3）的 FAIL 你判为**结构性缺陷**还是**测量/基座伪影**？依据是什么？
- Q2 若选②：E 议题范围是否以上述三件为准？还有必修项吗？
- Q3 若选①：需要什么"终局保护条款"（防止第四次翻案）？
- Q4 当前证据链（预注册+commit 标注+back-half 一致）是否已具备终局裁决力？

## 四、代码级审查重点（至少一家做，行号级证据）

1. `crates/sim-traffic/src/lib.rs`：fit_pnorm_concave_detailed / evaluate_q_fn / family_verdict 的判定链
2. `crates/sim-traffic/examples/mfd_sete.rs`：预注册 fixture 是否被忠实执行（种子段/拓扑/选族顺序）
3. `docs/benchmarks/mfd-3a-sete.txt`：与源码口径一致性（n 计数、back-half、预选 emit）
4. 32×32 free 区样本构成与 ‰ 取整舍入的影响量级

## 五、裁决格式（请原样返回）

```
## 决策6 裁决记录
- 署名：（具体型号 / GitHub 昵称）
- 选项：① / ② / 都不选（附理由）
- Q1-Q4 逐项意见：
- 代码级审查发现（如有）：
- 风险/附加条件：
```

## 六、材料清单

任务书、ADR-NEW-25（含 Set E 节）、ADR-NEW-26、mfd-3a-sete.txt（证据）、
mfd_sete.rs（预注册 fixture）、sim-traffic lib.rs（判定链）、decision-log（#31 全史）、
MANIFEST（口径说明）。


---

## FILE: adr/ADR-NEW-25-MFD交通模型与验证方案.md

# ADR-NEW-25：MFD 交通模型与验证方案

- 编号：ADR-NEW-25（暂定，入仓正式重编号时保留映射）
- 状态：Proposed（草案，待第 7 章评审流程正式裁决）
- 日期：2026-11-19
- 关联主文档章节：2.6（交通流量模型）、2.1（MFD 行）、Stage 2C（MFD 模型验证、交通模型可复现验证场景）
- 来源：Stage 2C 实施（C1）；设计审计"验证只有一句话无场景无指标"处置
- 裁决人：项目负责人（可委托外部评审，遵循 review/REVIEW-CONVENTIONS.md）

## 背景

主文档将 MFD（Macroscopic Fundamental Diagram，宏观基本图）列为交通流量模拟候选模型
（2.6："交通流量模型以 MFD 为候选设计方向。MFD 的适用范围、参数校准、与微观 Agent
交通行为之间的耦合方式，需要通过独立测试确认"），但全文没有验证场景、指标与判据。
设计审计与外部评审均指出：无场景无指标的"待验证"无法执行。本 ADR 给出验证方案本身
（模型形式仍为候选，参数仍待校准）。

## 问题

1. MFD 曲线形式未定（三角基本图 vs 二次凹曲线 vs 分段拟合）。
2. "适用范围、参数校准、耦合方式"三项要求（2.6）无验证场景与判据。
3. MFD 输出是否反馈权威模拟（影响路径选择/降级）未定——反馈则必须满足 2.9 确定性边界。
4. 交通模型本身（sim-traffic）无任何实现，"交通模型具备可复现验证场景"（Stage 2C 退出条件）
   无从谈起。

## 决策

1. **模型形式（2026-11-19 决策4 投票定案：凹形饱和扩展，A7:B2:C1）**：区域级基本图
   q(k) 的自由流段改为**凹形饱和分支**（不再假设线性自由流）：
   - **函数族写死（不得停留候选括号）**：单调凹饱和族 q(0)=0、零密度斜率自由估计
     但**越界判失败**（物理上界 1000‰；禁止"把约束当估计"——豆包）、渐近可估、
     纯幂律 q=c·k^p（p<1）**排除**（k→0 导数发散，豆包/Kimi）。默认候选=**有理饱和式
     q=q_max·k/(k+k_half)**（定点友好 +−×÷ 可实现；多家复算 C 各点 15–83‰），备选=
     定点对数/指数饱和（超越函数须定点实现保确定性红线，MiMo/Claude）。
   - **k_turn 显式衔接参数保留**（B 家意见吸收，DeepSeek/Kimi/Grok）：自由流凹段与
     平台段经显式衔接参数连接，禁止两段硬接（硬接会退化为现平台型复现缺陷）。
   - 可选下降支保留（`descending_branch_observed` 语义迁移到新族）。
   - 拟合=族内**自由估计**+定点实现（i64/i128、舍入平局规则入黄金测试）。
   - 参数仍由决策 3 校准流程从微观基准采样得出。
   - **决策5 投票修订（2026-11-21，Q2 采纳 2/有条件 8/延后 1）**：默认候选族=
     **pnorm-p2**（q=q_max·k/√(k²+k0²)，定点 isqrt 就近舍入；黄金已钉+floor 偏差披露）；
     MiMo 异议记录：pnorm 是"淘汰赛冠军"——拟合集 rel-SSE 劣于有理族、转折区误差更大，
     胜出靠物理可行性（binding 语义）非拟合优度。转定案条件=isqrt 黄金✅、拟合集逐点
     归档✅、back-half✅、**Set E 确认轮全过（未达成：2026-11-21 2/3，32×32 free 133‰）**。
   （依据：决策4 十家投票；根因=点队列 q(k) 凹形、平台型/三角型线性自由流假设失效。
   少数异议如实保留：MiniMax-C/E（基准无交通物理语义，拟合的是产物非定律，应先修
   微观基准）、Claude 基准漂移疑云（已核对 main 当前 mfd-tbench1.txt v_free=1000‰，
   疑云不成立）。）
2. **MFD 的权威性边界（2026-11-19 投票修订：三层拆分）**：
   - 微观交通状态（sim-traffic 车辆/位置/容量记账）= **权威状态**（入档入摘要）；
   - MFD 样本 (k,q) 与拟合参数 = **非权威派生统计**（仅展示/校准用途）；
   - 降级等级（BudgetRegistry）= **权威状态**，但触发只依赖确定性工作量单位
     （ADR-NEW-14 投票定案口径）——MFD 信号若接入降级输入，必须经确定性定点口径
     并入权威路径并走"注入延迟/线程数不改变摘要"验证，且另行 ADR 修订。
   不直接反馈权威路径选择；反馈需另行 ADR。理由：反馈回路会把 MFD 参数误差放大进
   权威状态，且 2.6 明令"不得在缺少验证的情况下将 MFD 的理论表现直接当作实际引擎性能"。
   （投票依据：千问Go!/MiniMax 指出"降级等级是权威状态"原措辞自相矛盾。）
3. **三项独立验证**（2.6 原文要求，逐项给场景+指标+判据）：
   - 3a 适用范围（2026-11-19 投票修订：独立参照防同义反复）：MFD 聚合与微观实测同源
     恒等不构成验证。判据须基于**独立参照**：①人工合成精确基本图（已知参数注入→拟合
     恢复，见 sim-traffic `synthetic_triangle_is_recovered`）；②留出验证集（拟合与
     评估用不同密度档）；③（二期）CTM/LWR 解析模型对照。判据=自由流区相对误差 ≤10%、
     临界密度位置偏差 ≤20%、拥堵区误差 ≤25%（阈值为占位候选，校准后复核）。
     （投票依据：MiniMax/千问Go!/Claude——"循环验证/同义反复"。）
     **2026-11-19 校准复核（V1 实测，docs/benchmarks/mfd-3a-reference.txt）**：
     判据三区分立定案=自由流 k<k_turn/2 ≤10%、饱和区 k≥4·k_turn ≤25%、k_c ≤20%
     （仅三角型；平台型 N/A 无峰值）、转折区只记录；三集协议=拟合 A / 试点 B（区域
     边界导出，披露口径）/ 确认 C（裁决集）+多 seed 平均（N=8，非权威统计方差缩减）。
     确认集实测：饱和区 102‰ **通过**、自由流区 190‰ **未通过**（判据 100‰）、
     转折区 521‰ 记录。未通过根因=点队列 q(k) 自 k≈34–54‰ 起即凹形、无独立线性自由流
     段（A 组 (76,67) 已偏 134‰、(205,135) 偏 519‰（Claude 口径勘误）），属**模型形式**缺陷——平台型/三角型
     均假设线性自由流分支。处置=决策4 投票（A7:B2:C1）凹形扩展已定案（见决策 1）。原模型形式扩展条目（凹形自由
     流分支）立债务 #31 已由决策4 裁决处置=凹形扩展启动，待 Set D 复测完成后核销。
     **2026-11-19 决策4 判据修订（去自指，8 家共识）**：(a) 判据与模型解耦——分区
     边界 k_turn 由固定 fixture 预注册（复测前声明，禁止由被评模型参数导出；MiniMax
     敏感性实证 k_turn/4 下 190→48‰ 全过=同参 gaming）；(b) 转折区设硬判据 ≤250‰
     （521‰ 原只记录恰豁免最坏点）；(c) 复测协议=凹形族预注册+新确认集 Set D（种子
     段与 A/B/C 不相交；C 已参与 V1 判定耗损）+第二拓扑（24×24 或 64×64）+多 seed
     逐 seed 披露；(d) 各区样本数下限防虚假 PASS；(e) 复测报最差点非均值；(f) 判据
     常量单一来源（代码断言与 ADR 判据引用同一常量——门禁脱钩已立整改）。终止条件：
     Set D 或第二拓扑任一 FAIL → 转 C（放弃 MFD→降级）关闭，不再延后。3a 转 Verified 条件仍未清（如实记录）。
   - **Set D 复测结果（2026-11-20，预注册 fixture 执行完毕）**：两拓扑自由流判据均 FAIL
     （40×40：自由流最大误差 148‰ > 100‰；64×64：142‰ > 100‰；转折 70/137‰、饱和 28/44‰
     均 PASS）→ **按预注册终止条件裁决：转 C（放弃 MFD→降级输入），不再延后**。附带证据：
     自由估计零密度斜率 1242/1271‰ 超物理界 1000‰（约束绑定回退后仍 FAIL）——微观基准
     初始斜率 >1 车/边/tick 的现象本身即 MiniMax"基准无交通物理语义（E 议题）"的实证，
     E 项修基（车辆在途/路段存储、有向边独立容量）若未来立项，可重新评估本判定。
     稳态诊断：持续注入下高密度档不稳态（40×40 仅 1/8、64×64 0/8 档稳态，已披露）。
     数据文件：docs/benchmarks/mfd-3a-setd.txt。
   - 3b 参数校准：v_free/k_c/q_max/w 按决策 1 拟合口径（最大割线+下降支最小二乘，
     定点实现）产生，拟合输入输出全部落 docs/benchmarks/（T-Bench-1 已输出样本表+拟合值）；
     同种子重校准必须逐位一致（确定性约束）。
   - 3c 耦合方式（2026-11-19 投票修订：明确代码路径）：MFD→降级/背压输入路径
     （如区域密度超 k_c 触发降级信号）须有**生产代码路径**（当前 BudgetRegistry 尚无
     生产调用方——条件未清不得开工该模块）+专用"MFD→降级信号"线程无关测试
     （通用并行证据不替代耦合验证）。验证口径=Stage 2B chaos 模式（注入延迟/线程数
     变化不改变摘要）。
   - 3a/3b/3c 各自产出独立指标文件入 docs/benchmarks/mfd-*.txt（GPT-6）。
4. **可复现验证场景（交通基准场景 T-Bench-1；2026-11-19 与实装一致化）**：40×40 双向
   网格路网、固定种子 0x00C0_FFEE_1234_5678、OD 对由 sim-rng event_u64 派生、车辆
   按密度阶梯持续注入（2→256 倍增 8 档）、每档 260 tick、**采样=末 60 tick 窗口均值**
   （稳态判据：窗口前后半 q 差 ≤2‰）、上升/下降双向扫描（滞回判据：同注入点
   |q_up−q_down| ≤2‰）。场景全部参数进 fixture（sim-test-support 口径），逐 tick 折叠
   GlobalDigest（覆盖全部权威状态含 goal/next_id/pending——投票修复项）。
5. **确定性判据**：同种子两跑逐点 (k, q) 序列一致（GlobalDigest 相等）；
   注入顺序无关性（同一 ID 集不同注入顺序→终态摘要一致；已实装
   `explicit_id_injection_is_order_independent`，ID 由集中分配点分配=ADR-NEW-20 口径）。

## 考虑过的替代方案

| 备选 | 处置 | 理由 |
| --- | --- | --- |
| 二次凹曲线 q(k) | 备选 | 拟合自由度高但外推危险；三角基线更保守 |
| MFD 反馈路径选择 | 否决（默认） | 参数误差放大进权威状态；2.6 验证前禁止 |
| 纯统计不建模 | 否决 | 无法支撑降级输入与"交通流量模拟"定位 |
| 直接采用 CS 系列交通口径 | 否决 | 无公开数据可复现；T-Bench-1 自建基准 |

## 影响

- sim-traffic（当前空骨架）获得最小实现范围：微观基准（车辆沿最短路行驶+路段容量排队）
  +区域聚合采样器+基本图拟合（定点）。
- EC-01/EC-04：降级信号增加"区域密度"输入源（须走确定性工作量单位口径，见 ADR-NEW-14）。
- 债务 #6（坐标/单位/角度）部分收窄：密度单位=辆/格（网格路网），流量单位=辆/tick。

## 验证方法

1. T-Bench-1 场景实现并通过确定性判据（决策 5）——已实装（sim-traffic 4+5 测试、
   tbench1 双向扫描、逐 tick GlobalDigest）。
2. 3a/3b/3c 三项各自产出指标文件入 docs/benchmarks/mfd-*.txt（当前 tbench1 数据见
   docs/benchmarks/mfd-tbench1.txt）。
3. 3a 独立参照、3c 生产代码路径为转 Verified 的条件（条件未清不得开工对应模块）。

## 3a 复测结果（2026-11-20，X1-X4 整改后；证据 commit ce4a705）

- 口径修复（k/q 同群体、预注册绝对带 ZoneBands::SET_D{100,2000}、越界判失败、
  目标判据同构）后重跑 Set D，**判定翻转**：
  - 40×40：rational 三区 83/118/56‰ 全 PASS（自由估计斜率 1095‰ 越界→族淘汰）；
    **pnorm-p2 三区 83/189/12‰ 全 PASS**（斜率 932‰ 物理可行）。
  - 64×64：rational 三区 80/112/89‰ 全 PASS（越界 1092‰→族淘汰）；
    **pnorm-p2 三区 48/120/20‰ 全 PASS**（斜率 944‰）。
  - 原"自由流 148/142‰ FAIL"系 q>k 测量 bug 的误差地板（豆包复算 |7−6|/7=142.86‰
    恰为旧 FAIL 值），修复后消失。
- 结论：**"凹形不可救→转 C"不成立**——存在物理可行凹形族（p-norm p=2）两拓扑三区
  全过。按豆包 A2 预注册路径：转 C 终局回滚，凹形扩展继续，**重新投票确认**
  （转 C 回滚 / pnorm-p2 入决策 1 / 族预选规则=物理可行域内拟合集 rel-SSE 最小）。
- 遗留：pnorm back-half 稳态敏感性未单跑（rational back-half 与全窗结论一致）；
  稳态诊断两拓扑 0-1/8 档（持续注入固有，已披露）。

## 正式裁决（2026-11-19，决策2 投票）

- **裁决：有条件采纳 → Accepted（附条件）**（10/10 票有条件采纳，零拒绝零延后；
  裁决人=项目负责人组织的 10 家署名投票，维护者代录）。
- 理由：验证方案框架（三项独立验证+T-Bench-1+确定性判据）方向正确且已实装落地；
  投票共识的 11 项硬伤中代码面 5 项已修复并回归（拟合重写、fold_digest 补字段、
  窗口均值采样、双向扫描、注入顺序无关性），协议面已修订（模型形态辨识二选一、
  权威性三层拆分、3a 独立参照、3c 代码路径条件、协议与实装一致化）。
- 验证条件（转 Verified 前置，未清不得开工对应模块）：
  1. 3a 独立参照验证（人工合成图+留出验证集，二期 CTM/LWR 对照）完成并回填误差阈值；
  2. 3c 生产代码路径（MFD→降级信号）实装+专用耦合线程无关测试通过；
  3. 拟合阈值/参数经校准复核回填（当前 10%/20%/25% 为占位）；
  4. R3/R4 与并发场景下 MFD 采样口径复验（豆包保留票、DeepSeek 附条件）。
- 修订记录：本节决议含 2026-11-19 决策2 投票 10 家裁决要求的全部协议面修订；
  代码面修复见 crates/sim-traffic、crates/sim-core/budget.rs、crates/sim-network
  对应提交与回归测试。

## 3a Set E 确认轮（2026-11-21；预注册 fixture 9a73ee0，证据 mfd-3a-sete.txt）

- fixture：新种子段 0x00C0_FFEE_3333_0000（fit +0..7 / holdout +8..15 种子不共用）、
  拟合/留出阶梯交错不相交、拓扑 40×40 / **32×32（第三拓扑）** / 64×64、族与选族规则
  继承冻结实现（先落盘后取数）。
- 结果：40×40 pnorm-p2 83/190/12 PASS；**32×32 133/148/46 FAIL（自由流 133‰ > 100‰）**；
  64×64 48/120/20 PASS。三拓扑预选均按 Q3 规则选中 pnorm-p2（rational 全
  binding=true 被正确排除——预选修复生效）。back-half pnorm 40×40 66/120/20 PASS。
- 判定：**Set E 未全过（2/3 拓扑），凹形扩展（pnorm-p2 单族）未获确认轮通过**。
  按预注册纪律不得事后调判据/减样本救场；32×32 free 区 n=3 小样本与 ‰ 取整舍入
  敏感性（Claude/豆包精度异议）如实披露。处置候选（待负责人裁决）：①追认转 C；
  ②MiniMax E 议题（微观基准交通物理语义修基）后扩族再战再投。
- 稳态诊断：0-1/8 档稳态（持续注入固有，已披露）。

## 当前状态

- **Accepted（附条件）**（2026-11-19 决策2 投票裁决；条件清单见上节）。
- 实施面：T-Bench-1 已实装并通过确定性判据（Stage 2C 退出核对 6/6）；3a/3c 条件
  为转 Verified 前置。后续：独立参照验证 → 阈值回填 → 转 Verified。
- **转 Verified 条件进展（2026-11-19 V1–V5 实测）**：
  - 3a 独立参照验证：**部分通过**——饱和 102‰ PASS、自由流 190‰ FAIL（判据 100‰），
    模型形式凹形缺陷立债务 #31；处置=债务 #31 裁决前 MFD 仅统计展示不作降级触发依据
    （证据 docs/benchmarks/mfd-3a-reference.txt）。
  - 3c 生产代码路径+耦合验证：**半达成**——需求压力通道已实装（ADR-NEW-26 草案）
    + mfd_coupling 三路摘要一致（1/2/8 线程、混沌注入、串行参考）；触发收益验证
    随债务 #31。
  - 阈值校准回填：**候选已出**——债务 #27 三场景基线实测、候选判据=单次尝试展开
    >4096 或让步≥8 切机制 A，数值随 G2 定案（docs/benchmarks/G5-threshold-baseline.txt）。
  - R3/R4 复验：R3 已过（Stage 2B）；R4 移交发布门禁。
- **终局（2026-11-20 Set D 复测 FAIL → 预注册终止条件触发）：转 C**——MFD 永久仅作
  非权威统计展示，**不作降级/背压触发依据**；本 ADR 的"模型扩展"部分（凹形饱和族）已
  实装并通过合成恢复测试，但真实点队列数据两拓扑自由流判据均 FAIL（见上节 Set D 结果），
  按终止条件关闭、不再延后。MFD→降级解禁的唯一重开路径=MiniMax E 议题（微观基准交通
  物理语义修基）立项后重新验证，且须重新走投票裁决。ADR-NEW-26 需求压力通道同步冻结
  （见其当前状态）。


---

## FILE: adr/ADR-NEW-26-需求压力降级信号通道.md

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


---

## FILE: evidence/mfd-3a-sete.txt

   Compiling sim-core v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-core)
   Compiling sim-traffic v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-traffic)
    Finished `release` profile [optimized] target(s) in 0.89s
     Running `target\release\examples\mfd_sete.exe`
Set D re-verification (debt #31; pre-registered fixture, disjoint seeds)
evidence commit: 9a73ee0743c568bb050c1bbf6c6ae272d3f213e2
criteria: free<=100‰ transition<=250‰ saturated<=250‰ (single source = sim-traffic consts)
=== topology 40x40 (Set E seeds 0xc0ffee33330000+0..8) ===
steady diagnostics: fit 0/8 steady, holdout 1/8 steady (dual criterion; unsteady ladders disclosed)
fit: q_max=271‰ k_half=271‰ zero-density slope=1000‰ (free estimate 1099‰, bound binding: true)
  [fit-set rational] k     7‰ q_obs    7‰ q_model    7‰ err 0‰
  [fit-set rational] k    16‰ q_obs   16‰ q_model   15‰ err 62‰
  [fit-set rational] k    34‰ q_obs   32‰ q_model   30‰ err 62‰
  [fit-set rational] k    78‰ q_obs   67‰ q_model   61‰ err 89‰
  [fit-set rational] k   211‰ q_obs  135‰ q_model  119‰ err 118‰
  [fit-set rational] k   938‰ q_obs  211‰ q_model  210‰ err 4‰
  [fit-set rational] k  3092‰ q_obs  239‰ q_model  249‰ err 41‰
  [fit-set rational] k  7719‰ q_obs  248‰ q_model  262‰ err 56‰
  [fit-set pnorm-p2] k     7‰ q_obs    7‰ q_model    7‰ err 0‰
  [fit-set pnorm-p2] k    16‰ q_obs   16‰ q_model   15‰ err 62‰
  [fit-set pnorm-p2] k    34‰ q_obs   32‰ q_model   31‰ err 31‰
  [fit-set pnorm-p2] k    78‰ q_obs   67‰ q_model   70‰ err 44‰
  [fit-set pnorm-p2] k   211‰ q_obs  135‰ q_model  154‰ err 140‰
  [fit-set pnorm-p2] k   938‰ q_obs  211‰ q_model  239‰ err 132‰
  [fit-set pnorm-p2] k  3092‰ q_obs  239‰ q_model  247‰ err 33‰
  [fit-set pnorm-p2] k  7719‰ q_obs  248‰ q_model  248‰ err 0‰
  [family pre-selection] fit-set rel-SSE rational=34366 pnorm=44854 binding(r=true,p=false) -> selected=pnorm-p2 (Q3 feasible-domain rule; locked before holdout)
  k    12‰ q_obs   12‰ q_model   11‰ err 83‰
  k    25‰ q_obs   24‰ q_model   23‰ err 41‰
  k    55‰ q_obs   50‰ q_model   46‰ err 80‰
  k   132‰ q_obs  101‰ q_model   89‰ err 118‰
  k   502‰ q_obs  184‰ q_model  176‰ err 43‰
  k  1974‰ q_obs  231‰ q_model  238‰ err 30‰
  k  5385‰ q_obs  245‰ q_model  258‰ err 53‰
  k 12408‰ q_obs  250‰ q_model  265‰ err 60‰
  bound violation: free estimate slope 1099‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)
  free-flow max err   =   83‰  (criterion <= 100‰)  PASS
  transition max err  =  118‰  (criterion <= 250‰)  PASS
  saturated max err   =   60‰  (criterion <= 250‰)  PASS
  topology verdict [rational] = FAIL (binding=true free=83‰(n=3) trans=118‰(n=3) sat=60‰(n=2))
  [family pnorm-p2] fit: q_max=248‰ k0=266‰ slope free=932‰ binding=false
  [family pnorm-p2] free=83‰ (n=3) trans=190‰ (n=3) sat=12‰ (n=2) -> PASS
  Set D outcome (selected family pnorm-p2) = PASS
  [sensitivity back-half rational] free=83‰(n=3) trans=128‰(n=2) sat=68‰(n=3) binding=true -> FAIL
  [sensitivity back-half pnorm-p2] free=83‰(n=3) trans=194‰(n=2) sat=60‰(n=3) binding=false -> PASS
=== topology 32x32 (Set E seeds 0xc0ffee33330000+0..8) ===
steady diagnostics: fit 1/8 steady, holdout 1/8 steady (dual criterion; unsteady ladders disclosed)
fit: q_max=269‰ k_half=269‰ zero-density slope=1000‰ (free estimate 1092‰, bound binding: true)
  [fit-set rational] k    10‰ q_obs    9‰ q_model   10‰ err 111‰
  [fit-set rational] k    20‰ q_obs   20‰ q_model   19‰ err 50‰
  [fit-set rational] k    45‰ q_obs   42‰ q_model   39‰ err 71‰
  [fit-set rational] k   105‰ q_obs   85‰ q_model   76‰ err 105‰
  [fit-set rational] k   357‰ q_obs  166‰ q_model  153‰ err 78‰
  [fit-set rational] k  1737‰ q_obs  227‰ q_model  233‰ err 26‰
  [fit-set rational] k  5258‰ q_obs  245‰ q_model  256‰ err 44‰
  [fit-set rational] k 12595‰ q_obs  251‰ q_model  263‰ err 47‰
  [fit-set pnorm-p2] k    10‰ q_obs    9‰ q_model    9‰ err 0‰
  [fit-set pnorm-p2] k    20‰ q_obs   20‰ q_model   18‰ err 100‰
  [fit-set pnorm-p2] k    45‰ q_obs   42‰ q_model   39‰ err 71‰
  [fit-set pnorm-p2] k   105‰ q_obs   85‰ q_model   87‰ err 23‰
  [fit-set pnorm-p2] k   357‰ q_obs  166‰ q_model  196‰ err 180‰
  [fit-set pnorm-p2] k  1737‰ q_obs  227‰ q_model  248‰ err 92‰
  [fit-set pnorm-p2] k  5258‰ q_obs  245‰ q_model  251‰ err 24‰
  [fit-set pnorm-p2] k 12595‰ q_obs  251‰ q_model  251‰ err 0‰
  [family pre-selection] fit-set rel-SSE rational=41792 pnorm=57010 binding(r=true,p=false) -> selected=pnorm-p2 (Q3 feasible-domain rule; locked before holdout)
  k    15‰ q_obs   15‰ q_model   14‰ err 66‰
  k    33‰ q_obs   31‰ q_model   29‰ err 64‰
  k    72‰ q_obs   63‰ q_model   57‰ err 95‰
  k   188‰ q_obs  127‰ q_model  111‰ err 125‰
  k   951‰ q_obs  209‰ q_model  210‰ err 4‰
  k  3457‰ q_obs  239‰ q_model  250‰ err 46‰
  k  8906‰ q_obs  248‰ q_model  261‰ err 52‰
  k 19989‰ q_obs  252‰ q_model  265‰ err 51‰
  bound violation: free estimate slope 1092‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)
  free-flow max err   =   95‰  (criterion <= 100‰)  PASS
  transition max err  =  125‰  (criterion <= 250‰)  PASS
  saturated max err   =   52‰  (criterion <= 250‰)  PASS
  topology verdict [rational] = FAIL (binding=true free=95‰(n=3) trans=125‰(n=2) sat=52‰(n=3))
  [family pnorm-p2] fit: q_max=251‰ k0=286‰ slope free=877‰ binding=false
  [family pnorm-p2] free=133‰ (n=3) trans=148‰ (n=2) sat=46‰ (n=3) -> FAIL
  Set D outcome (selected family pnorm-p2) = FAIL
  [sensitivity back-half rational] free=96‰(n=3) trans=134‰(n=2) sat=20‰(n=3) binding=true -> FAIL
  [sensitivity back-half pnorm-p2] free=133‰(n=3) trans=152‰(n=2) sat=46‰(n=3) binding=false -> FAIL
=== topology 64x64 (Set E seeds 0xc0ffee33330000+0..8) ===
steady diagnostics: fit 1/8 steady, holdout 0/8 steady (dual criterion; unsteady ladders disclosed)
fit: q_max=284‰ k_half=284‰ zero-density slope=1000‰ (free estimate 1092‰, bound binding: true)
  [fit-set rational] k     4‰ q_obs    4‰ q_model    4‰ err 0‰
  [fit-set rational] k     9‰ q_obs    9‰ q_model    9‰ err 0‰
  [fit-set rational] k    20‰ q_obs   20‰ q_model   19‰ err 50‰
  [fit-set rational] k    44‰ q_obs   41‰ q_model   38‰ err 73‰
  [fit-set rational] k   101‰ q_obs   83‰ q_model   75‰ err 96‰
  [fit-set rational] k   292‰ q_obs  157‰ q_model  144‰ err 82‰
  [fit-set rational] k   989‰ q_obs  217‰ q_model  221‰ err 18‰
  [fit-set rational] k  2704‰ q_obs  239‰ q_model  257‰ err 75‰
  [fit-set pnorm-p2] k     4‰ q_obs    4‰ q_model    4‰ err 0‰
  [fit-set pnorm-p2] k     9‰ q_obs    9‰ q_model    9‰ err 0‰
  [fit-set pnorm-p2] k    20‰ q_obs   20‰ q_model   19‰ err 50‰
  [fit-set pnorm-p2] k    44‰ q_obs   41‰ q_model   41‰ err 0‰
  [fit-set pnorm-p2] k   101‰ q_obs   83‰ q_model   89‰ err 72‰
  [fit-set pnorm-p2] k   292‰ q_obs  157‰ q_model  181‰ err 152‰
  [fit-set pnorm-p2] k   989‰ q_obs  217‰ q_model  232‰ err 69‰
  [fit-set pnorm-p2] k  2704‰ q_obs  239‰ q_model  239‰ err 0‰
  [family pre-selection] fit-set rel-SSE rational=29718 pnorm=35549 binding(r=true,p=false) -> selected=pnorm-p2 (Q3 feasible-domain rule; locked before holdout)
  k     7‰ q_obs    7‰ q_model    7‰ err 0‰
  k    15‰ q_obs   14‰ q_model   14‰ err 0‰
  k    33‰ q_obs   31‰ q_model   30‰ err 32‰
  k    71‰ q_obs   62‰ q_model   57‰ err 80‰
  k   179‰ q_obs  124‰ q_model  110‰ err 112‰
  k   606‰ q_obs  199‰ q_model  193‰ err 30‰
  k  1826‰ q_obs  233‰ q_model  246‰ err 55‰
  k  4491‰ q_obs  245‰ q_model  267‰ err 89‰
  bound violation: free estimate slope 1092‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)
  free-flow max err   =   80‰  (criterion <= 100‰)  PASS
  transition max err  =  112‰  (criterion <= 250‰)  PASS
  saturated max err   =   89‰  (criterion <= 250‰)  PASS
  topology verdict [rational] = FAIL (binding=true free=80‰(n=4) trans=112‰(n=3) sat=89‰(n=1))
  [family pnorm-p2] fit: q_max=240‰ k0=254‰ slope free=944‰ binding=false
  [family pnorm-p2] free=48‰ (n=4) trans=120‰ (n=3) sat=20‰ (n=1) -> PASS
  Set D outcome (selected family pnorm-p2) = PASS
  [sensitivity back-half rational] free=80‰(n=4) trans=112‰(n=3) sat=93‰(n=1) binding=true -> FAIL
  [sensitivity back-half pnorm-p2] free=66‰(n=4) trans=120‰(n=3) sat=20‰(n=1) binding=false -> PASS


---

## FILE: src/mfd_sete.rs

//! Set D 复测（债务 #31 核销条件；ADR-NEW-25 3a 决策4 判据修订 (c)）。
//! Set E 确认轮（决策5 收口条件）：新种子段 0x00C0_FFEE_3333_0000、
//! fit/holdout 种子不共用（fit +0..7 / holdout +8..15）、第三拓扑 32x32、
//! 族集合与选族规则已于 c0f94ba 冻结——本文件先落盘后取数（预注册）。
//!
//! 预注册 fixture（**运行前声明，先于 Set D 数据**）：
//! - Set D 种子段=0x00C0_FFEE_2222_0000 + i（i=0..8），与 V1 的 A/B/C 段
//!   （0x00C0_FFEE_1234_5678+i）不相交；
//! - 拓扑：主 40×40 + 第二拓扑 64×64（判据修订要求）；
//! - 密度阶梯：拟合集 2→256 倍增 8 档；留出集 3→384 倍增 8 档（交错不相交）；
//! - 每档 260 tick，末 60 tick 窗口均值；稳态判据=窗口前后半 q 差 ≤2‰；
//! - 每档 8 种子平均；分区边界=ZoneBands::SET_D 预注册绝对带（k<100 / 100..2000 /
//!   ≥2000‰，禁由拟合参数导出——审计修复 2026-11-20，决策4 3a(a)）；
//! - 判据（常量单一来源=sim-traffic）：自由流 ≤100‰、转折 ≤250‰、饱和 ≤250‰。
//!
//! 终止条件（Claude）：任一拓扑任一区 FAIL → 转 C，不再延后。
//!
//! 运行：cargo run --release --example mfd_setd -p sim-traffic

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use sim_traffic::{
    evaluate_pnorm_fit, evaluate_rational_fit, fit_pnorm_concave_detailed,
    fit_rational_saturation_detailed, q_pnorm_permille, q_rational_permille, FitErrorReport,
    GridTraffic,
    MfdSample, ZoneBands, FREE_FLOW_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE,
};

const SETD_SEED_BASE: u64 = 0x00C0_FFEE_3333_0000;
const N_SEEDS: u64 = 8;
const TICKS: usize = 260;
const WINDOW: usize = 60;
const FIT_LADDER: [u64; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
const HOLDOUT_LADDER: [u64; 8] = [3, 6, 12, 24, 48, 96, 192, 384];

/// 单运行：返回 (k̄, q̄) 窗口均值与稳态判定。
fn run_density_seed(w: u32, h: u32, inj: u64, seed: u64) -> (u64, u64, bool) {
    let mut gt = GridTraffic::new(w, h, 1, seed);
    let mut k_sum = 0u64;
    let mut q_sum = 0u64;
    let mut q_front = 0u64;
    let mut q_back = 0u64;
    for t in 0..TICKS {
        gt.inject(inj);
        gt.step();
        if t >= TICKS - WINDOW {
            // q≤k fail-fast（审计修复 2026-11-20）：同群体口径（last_step_population
            // 含到达车末步，与 flow 同群体）下每车每 tick ≤1 条边。
            assert!(
                gt.flow_this_tick <= gt.last_step_population,
                "q>k impossible under same-population accounting"
            );
            k_sum += gt.last_step_population * 1000 / gt.edge_count();
            q_sum += gt.flow_this_tick * 1000 / gt.edge_count();
            if t >= TICKS - WINDOW / 2 {
                q_back += gt.flow_this_tick;
            } else {
                q_front += gt.flow_this_tick;
            }
        }
    }
    let n = WINDOW as u64;
    // 稳态判据双口径：相对差 ≤2‰，或绝对差 ≤ 半窗口一半（≈0.5 车/tick）——
    // 小流量点 1 辆车波动即超 2‰，纯相对口径是测量分辨率问题非发散（W2 实测教训）。
    let steady = q_front.abs_diff(q_back) * 1000 <= (q_front + q_back) * 2
        || q_front.abs_diff(q_back) <= (WINDOW as u64) / 4;
    (k_sum / n, q_sum / n, steady)
}

/// 稳态敏感性采样（审计修复 2026-11-20，DeepSeek A5/MiniMax A5）：仅窗口后半
/// （t >= TICKS − WINDOW/2）采样，返回 (k̄, q̄)。
fn run_density_seed_back_half(w: u32, h: u32, inj: u64, seed: u64) -> (u64, u64) {
    let mut gt = GridTraffic::new(w, h, 1, seed);
    let mut k_sum = 0u64;
    let mut q_sum = 0u64;
    for t in 0..TICKS {
        gt.inject(inj);
        gt.step();
        if t >= TICKS - WINDOW / 2 {
            k_sum += gt.last_step_population * 1000 / gt.edge_count();
            q_sum += gt.flow_this_tick * 1000 / gt.edge_count();
        }
    }
    let n = (WINDOW / 2) as u64;
    (k_sum / n, q_sum / n)
}

/// 多种子平均采样。稳态仅作诊断（不阻断）：返回 (样本, 未稳态档数)。
fn sample_ladder(w: u32, h: u32, ladder: &[u64], seed_off: u64) -> (Vec<MfdSample>, usize) {
    let mut unsteady = 0usize;
    let samples = ladder
        .iter()
        .map(|&inj| {
            let (mut k, mut q, mut steady) = (0u64, 0u64, true);
            for i in 0..N_SEEDS {
                let (k1, q1, s1) = run_density_seed(w, h, inj, SETD_SEED_BASE + seed_off + i);
                k += k1;
                q += q1;
                steady &= s1;
            }
            if !steady {
                unsteady += 1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    (samples, unsteady)
}

/// 相对误差平方和（‰口径）——族比较统一目标（与判据同构）。
/// 模型函数表项（clippy: type_complexity 收敛；生命周期须带出 'static 默认）。
type QModelFn<'a> = Box<dyn Fn(u64) -> u64 + 'a>;
fn rel_sse(
q: impl Fn(u64) -> u64, samples: &[MfdSample]) -> u128 {
    let mut sse = 0u128;
    for s in samples {
        let e = q(s.k_permille) as i128 - s.q_permille as i128;
        let er = (e * 1000) / (s.q_permille.max(1) as i128);
        sse += (er * er) as u128;
    }
    sse
}

/// 单一判定核（决策5 整改：合并原 family_verdict/topology_fail 双实现——
/// 双实现漂移教训）。越界（binding）即 FAIL（决策4 条件⑧）；三区判据常量
/// 单一来源，**含转折区硬判据**（决策4(b)——决策5 五家行号命中的漏接线缺陷）；
/// 空区即 FAIL（决策4(d) 样本数下限：防空区真空 PASS；<2 点区在输出行标 n 值披露）。
fn family_verdict(binding: bool, r: &FitErrorReport) -> &'static str {
    if binding
        || r.free_flow_max_err_permille > FREE_FLOW_CRIT_PERMILLE
        || r.transition_max_err_permille > TRANSITION_CRIT_PERMILLE
        || r.saturated_max_err_permille > SATURATED_CRIT_PERMILLE
        || r.free_n == 0
        || r.trans_n == 0
        || r.sat_n == 0
    {
        "FAIL"
    } else {
        "PASS"
    }
}

fn report_topology(w: u32, h: u32, emit: &mut dyn FnMut(String)) {
    emit(format!("=== topology {w}x{h} (Set E seeds {SETD_SEED_BASE:#x}+0..{N_SEEDS}) ==="));
    let (mut fit_samples, unsteady_fit) = sample_ladder(w, h, &FIT_LADDER, 0);
    fit_samples.sort_by_key(|s| s.k_permille);
    let (holdout, unsteady_hold) = sample_ladder(w, h, &HOLDOUT_LADDER, 8);
    emit(format!(
        "steady diagnostics: fit {}/{} steady, holdout {}/{} steady (dual criterion; unsteady ladders disclosed)",
        FIT_LADDER.len() - unsteady_fit,
        FIT_LADDER.len(),
        HOLDOUT_LADDER.len() - unsteady_hold,
        HOLDOUT_LADDER.len()
    ));
    let (fit, free_slope, binding) = fit_rational_saturation_detailed(&fit_samples);
    emit(format!(
        "fit: q_max={}‰ k_half={}‰ zero-density slope={}‰ (free estimate {}‰, bound binding: {})",
        fit.q_max_permille,
        fit.k_half_permille,
        fit.zero_density_slope_permille(),
        free_slope,
        binding
    ));
    // ===== 决策5 Y1：选族前置（Q3 实装）——先辨识两族、只用拟合集选族并锁定，
    // 之后才生成留出表与评估（计算与打印顺序一致，防"先见留出"误读，Kimi）。
    let (pfit, p_slope, p_binding) = fit_pnorm_concave_detailed(&fit_samples);
    // 拟合集逐点归档（决策5 MiniMax/豆包 R5：预选所依据的 rel-SSE 须可复核）。
    let qf_tab: Vec<(&str, QModelFn<'_>)> = vec![
        ("rational", Box::new(|k: u64| q_rational_permille(&fit, k))),
        ("pnorm-p2", Box::new(|k: u64| q_pnorm_permille(&pfit, k))),
    ];
    for (tag, qf) in &qf_tab {
        for s in &fit_samples {
            let qm = qf(s.k_permille);
            let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000 / (s.q_permille.max(1) as u128);
            emit(format!(
                "  [fit-set {tag}] k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ err {err}‰",
                s.k_permille, s.q_permille, qm
            ));
        }
    }
    let sse_r = rel_sse(|k| q_rational_permille(&fit, k), &fit_samples);
    let sse_p = rel_sse(|k| q_pnorm_permille(&pfit, k), &fit_samples);
    // Q3 预选（决策5 全票通过）：物理可行域（binding=false）内拟合集 rel-SSE 最小；
    // 越界族不具参赛资格（条件⑧族级读法：被判失败者不得当选）；全部越界→无族
    // 可选（族级 FAIL 兜底，Grok/Gemini）；一次锁定不得回退（豆包）。
    let selected: Option<&str> = match (binding, p_binding) {
        (true, true) => None,
        (false, true) => Some("rational"),
        (true, false) => Some("pnorm-p2"),
        (false, false) => Some(if sse_p < sse_r { "pnorm-p2" } else { "rational" }),
    };
    emit(format!(
        "  [family pre-selection] fit-set rel-SSE rational={sse_r} pnorm={sse_p} binding(r={binding},p={p_binding}) -> selected={} (Q3 feasible-domain rule; locked before holdout)",
        selected.unwrap_or("NONE (all binding)")
    ));
    // 留出集逐点表（证据展示；选族已锁定，不再影响 selected）
    for s in &holdout {
        let qm = q_rational_permille(&fit, s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000 / (s.q_permille.max(1) as u128);
        emit(format!(
            "  k {:>5}‰ q_obs {:>4}‰ q_model {:>4}‰ err {}‰",
            s.k_permille, s.q_permille, qm, err
        ));
    }
    let r = evaluate_rational_fit(&fit, &holdout, &ZoneBands::SET_D);
    let verdict = |v: u64, c: u64| if v <= c { "PASS" } else { "FAIL" };
    // 越界判失败（决策4 条件⑧，审计修复 2026-11-20）：自由估计越界→本拓扑直接
    // FAIL；binding 回退解仅诊断披露，不参与 PASS/FAIL 判定。
    if binding {
        emit(format!(
            "  bound violation: free estimate slope {}‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)",
            free_slope
        ));
    }
    emit(format!(
        "  free-flow max err   = {:>4}‰  (criterion <= {}‰)  {}",
        r.free_flow_max_err_permille, FREE_FLOW_CRIT_PERMILLE,
        verdict(r.free_flow_max_err_permille, FREE_FLOW_CRIT_PERMILLE)
    ));
    emit(format!(
        "  transition max err  = {:>4}‰  (criterion <= {}‰)  {}",
        r.transition_max_err_permille, TRANSITION_CRIT_PERMILLE,
        verdict(r.transition_max_err_permille, TRANSITION_CRIT_PERMILLE)
    ));
    emit(format!(
        "  saturated max err   = {:>4}‰  (criterion <= {}‰)  {}",
        r.saturated_max_err_permille, SATURATED_CRIT_PERMILLE,
        verdict(r.saturated_max_err_permille, SATURATED_CRIT_PERMILLE)
    ));
    // 单一判定核输出（含转折硬判据+空区 FAIL+各区 n 披露，决策5 整改）
    emit(format!(
        "  topology verdict [rational] = {} (binding={} free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}))",
        family_verdict(binding, &r), binding,
        r.free_flow_max_err_permille, r.free_n,
        r.transition_max_err_permille, r.trans_n,
        r.saturated_max_err_permille, r.sat_n
    ));
    // ===== pnorm 留出评估（选族已在前置块锁定，此处只评估+终局）=====
    emit(format!(
        "  [family pnorm-p2] fit: q_max={}‰ k0={}‰ slope free={}‰ binding={}",
        pfit.q_max_permille, pfit.k0_permille, p_slope, p_binding
    ));
    let rp = evaluate_pnorm_fit(&pfit, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "  [family pnorm-p2] free={}‰ (n={}) trans={}‰ (n={}) sat={}‰ (n={}) -> {}",
        rp.free_flow_max_err_permille, rp.free_n, rp.transition_max_err_permille, rp.trans_n,
        rp.saturated_max_err_permille, rp.sat_n,
        family_verdict(p_binding, &rp)
    ));
    emit(format!(
        "  Set E outcome (selected family {}) = {}",
        selected.unwrap_or("NONE"),
        match selected {
            Some("pnorm-p2") => family_verdict(p_binding, &rp),
            Some(_) => family_verdict(binding, &r),
            None => "FAIL (no feasible family: all binding)",
        }
    ));
    // 稳态敏感性（审计修复 2026-11-20）：窗口后半单独采样复算三区指标，
    // 若与全窗结论一致则"稳态仅披露"升级为"已验证稳健"。
    let back_fit: Vec<MfdSample> = FIT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + 8 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let back_hold: Vec<MfdSample> = HOLDOUT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + 8 + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let (fit_b, _, binding_b) = fit_rational_saturation_detailed(&back_fit);
    let r_b = evaluate_rational_fit(&fit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [sensitivity back-half rational] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        r_b.free_flow_max_err_permille, r_b.free_n,
        r_b.transition_max_err_permille, r_b.trans_n,
        r_b.saturated_max_err_permille, r_b.sat_n,
        binding_b,
        family_verdict(binding_b, &r_b)
    ));
    // 决策5 共识收口条件：pnorm back-half 敏感性（此前只测 rational）。
    let (pfit_b, _, p_binding_b) = fit_pnorm_concave_detailed(&back_fit);
    let rp_b = evaluate_pnorm_fit(&pfit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [sensitivity back-half pnorm-p2] free={}‰(n={}) trans={}‰(n={}) sat={}‰(n={}) binding={} -> {}",
        rp_b.free_flow_max_err_permille, rp_b.free_n,
        rp_b.transition_max_err_permille, rp_b.trans_n,
        rp_b.saturated_max_err_permille, rp_b.sat_n,
        p_binding_b,
        family_verdict(p_binding_b, &rp_b)
    ));
}

fn main() {
    let mut out: Vec<String> = Vec::new();
    out.push("Set D re-verification (debt #31; pre-registered fixture, disjoint seeds)".to_string());
    // 证据-源码同源标注（审计修复 2026-11-20）：证据文件自带生成它的 commit hash
    // （编译期捕获：运行前设 SETD_COMMIT=$(git rev-parse HEAD) 并重编译）。
    out.push(format!(
        "evidence commit: {}",
        option_env!("SETD_COMMIT").unwrap_or("(unset — build with SETD_COMMIT env)")
    ));
    out.push(format!(
        "criteria: free<={}‰ transition<={}‰ saturated<={}‰ (single source = sim-traffic consts)",
        FREE_FLOW_CRIT_PERMILLE, TRANSITION_CRIT_PERMILLE, SATURATED_CRIT_PERMILLE
    ));
    report_topology(40, 40, &mut |l| out.push(l));
    report_topology(32, 32, &mut |l| out.push(l));
    report_topology(64, 64, &mut |l| out.push(l));
    for l in &out {
        println!("{l}");
    }
}


---

## FILE: src/lib.rs

//! 交通系统：MFD 微观基准 + 区域聚合采样（ADR-NEW-25，Stage 2C C3b）。
//!
//! - 权威 crate。确定性：BTreeMap 队列、车辆按 ID 全序、event_u64 派生 OD、
//!   u64 tick 计数，无时钟/无浮点/无 HashMap（2.9）。
//! - MFD 三角基本图（候选）：微观基准采样 (k, q) → 定点拟合 v_free/k_c/q_max/w。
//! - T-Bench-1（ADR-NEW-25 决策 4）：40×40 双向网格、密度阶梯注入、每档稳定
//!   200 tick 后采样；本文件以缩小档位实现，example 输出完整档位。

use sim_codec::CanonicalWriter;
use sim_digest::GlobalDigest;
use sim_rng::event_u64;

pub const LAYER: &str = "交通系统";
pub const AUTHORITATIVE: bool = true;

pub fn authoritative() -> bool {
    AUTHORITATIVE
}

/// 车辆（确定性：ID 即全序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vehicle {
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub goal_x: u32,
    pub goal_y: u32,
    pub edge_traversals: u64,
}

impl Vehicle {
    pub fn arrived(&self) -> bool {
        self.x == self.goal_x && self.y == self.goal_y
    }
}

/// 网格交通基准（T-Bench-1）：W×H 双向网格，每边每 tick 容量 cap 辆。
#[derive(Debug, Clone)]
pub struct GridTraffic {
    pub w: u32,
    pub h: u32,
    pub cap_per_edge_per_tick: u64,
    pub tick: u64,
    /// 等待进入路网的注入队列（按 (注入 tick, id) 全序）。
    pending: Vec<Vehicle>,
    pub vehicles: Vec<Vehicle>,
    next_id: u64,
    /// 本 tick 完成的边遍历数（流量采样）。
    pub flow_this_tick: u64,
    /// 本 tick 参与移动的车辆数（step 中移除到达车**之前**的人口；k 采样口径，
    /// 与 flow 同群体 → q≤k 恒成立——审计修复 2026-11-20：旧口径 flow 含到达车
    /// 末步、active_count 不含，q>k 违规点伪造误差地板）。
    pub last_step_population: u64,
    seed: u64,
}

impl GridTraffic {
    pub fn new(w: u32, h: u32, cap: u64, seed: u64) -> Self {
        Self {
            w,
            h,
            cap_per_edge_per_tick: cap,
            tick: 0,
            pending: Vec::new(),
            vehicles: Vec::new(),
            next_id: 0,
            flow_this_tick: 0,
            last_step_population: 0,
            seed,
        }
    }

    /// 注入 n 辆车（OD 由 event_u64 确定性派生；曼哈顿路径，先 x 后 y）。
    pub fn inject(&mut self, n: u64) {
        for _ in 0..n {
            let id = self.next_id;
            self.next_id += 1;
            self.inject_with_id(id);
        }
    }

    /// 显式 ID 注入（2026-11-19 投票修复：注入顺序无关性）。
    /// 契约：ID 由集中分配点按稳定键分配（ADR-NEW-20 口径）；OD 派生只依赖 ID，
    /// pending 入档/进网均按 id 稳定序 → 注入的物理顺序不影响任何权威状态。
    /// 调用方负责 ID 唯一性（重复 ID 会产生两辆车，属调用方契约违例）。
    pub fn inject_with_id(&mut self, id: u64) {
        let sx = (event_u64(self.seed, "traffic.od", id, 0) % self.w as u64) as u32;
        let sy = (event_u64(self.seed, "traffic.od", id, 1) % self.h as u64) as u32;
        let gx = (event_u64(self.seed, "traffic.od", id, 2) % self.w as u64) as u32;
        let gy = (event_u64(self.seed, "traffic.od", id, 3) % self.h as u64) as u32;
        self.pending.push(Vehicle {
            id,
            x: sx,
            y: sy,
            goal_x: gx,
            goal_y: gy,
            edge_traversals: 0,
        });
    }

    /// 推进一个 tick：注入进网、车辆沿曼哈顿路径前进一步（容量让行）。
    /// 全序：车辆按 id 升序处理；同位置同 tick 内先到先行（id 序）。
    pub fn step(&mut self) {
        self.tick += 1;
        self.flow_this_tick = 0;
        // 注入进网（pending 已按 id 序）
        let entering: Vec<Vehicle> = std::mem::take(&mut self.pending);
        for v in entering {
            self.vehicles.push(v);
        }
        self.vehicles.sort_by_key(|v| v.id);
        // 容量记账：每个目标格每 tick 只放行 cap 辆（cap=1 时即逐格排队）
        let mut admitted_at: std::collections::BTreeMap<(u32, u32), u64> =
            std::collections::BTreeMap::new();
        for v in self.vehicles.iter_mut() {
            if v.arrived() {
                continue;
            }
            let (nx, ny) = if v.x != v.goal_x {
                (if v.x < v.goal_x { v.x + 1 } else { v.x - 1 }, v.y)
            } else {
                (v.x, if v.y < v.goal_y { v.y + 1 } else { v.y - 1 })
            };
            let slot = admitted_at.entry((nx, ny)).or_insert(0);
            if *slot < self.cap_per_edge_per_tick {
                *slot += 1;
                v.x = nx;
                v.y = ny;
                v.edge_traversals += 1;
                self.flow_this_tick += 1;
            }
        }
        // k 采样口径（审计修复 2026-11-20）：人口在移除到达车之前计量，与
        // flow_this_tick（含到达车末步）同群体 → q≤k 恒成立（每车每 tick ≤1 条边）。
        self.last_step_population = self.vehicles.len() as u64;
        // 已到达车辆出网（保持 vehicles=活跃车辆）
        self.vehicles.retain(|v| !v.arrived());
    }

    /// 活跃车辆数（密度分子）。
    /// 需求压力（万分比 0..=10000）= 当前密度 k 相对特征密度 k_ref_permille 的比值封顶。
    /// 确定性定点（权威状态导出）；MFD→降级信号通道（ADR-NEW-26、ADR-NEW-25 3c）。
    /// 决策4 修订：k_ref=特征密度（q 首次 ≥90%·q_max 密度，版本化）；k_ref==0 返回 0
    /// （fail-safe 改向：分母未知时宁可不触发降级，原返回 10000 方向反）。
    pub fn demand_pressure_bp(&self, k_ref_permille: u64) -> u32 {
        if k_ref_permille == 0 || self.edge_count() == 0 {
            return 0;
        }
        // 采样口径=last_step_population（与 q 同群体；审计修复 2026-11-20，
        // MiMo #5：乘法走 u128 防 u64 环绕；edge_count==0 fail-safe 返回 0）。
        let k = (self.last_step_population as u128 * 1000 / self.edge_count() as u128) as u64;
        ((k as u128 * 10_000 / k_ref_permille as u128).min(10_000)) as u32
    }

    pub fn active_count(&self) -> u64 {
        self.vehicles.len() as u64
    }

    /// 网格链接总数（双向边数；密度分母口径=边数）。
    pub fn edge_count(&self) -> u64 {
        (2 * self.w as u64 * (self.h as u64 - 1)) + (2 * self.h as u64 * (self.w as u64 - 1))
    }

    /// 逐 tick 折叠摘要（G3/确定性判据用）。
    /// 覆盖全部权威状态（ADR-NEW-19 纳入范围）：配置、tick、pending 注入队列、
    /// next_id 分配器、逐车全字段（含 goal 与 traversals）——2026-11-19 投票修复：
    /// 补 goal_x/goal_y/next_id 等字段，消除强读档等价假通过风险。
    pub fn fold_digest(&self, g: &mut GlobalDigest) {
        let mut w = CanonicalWriter::new("sim-traffic", "TrafficTick");
        w.u64(self.w as u64);
        w.u64(self.h as u64);
        w.u64(self.cap_per_edge_per_tick);
        w.u64(self.seed);
        w.u64(self.tick);
        w.u64(self.next_id);
        // pending 按 id 稳定序入档（注入顺序无关契约）
        let mut pending_sorted: Vec<&Vehicle> = self.pending.iter().collect();
        pending_sorted.sort_by_key(|v| v.id);
        w.seq_len(pending_sorted.len());
        for v in pending_sorted {
            w.u64(v.id);
            w.u32(v.x);
            w.u32(v.y);
            w.u32(v.goal_x);
            w.u32(v.goal_y);
            w.u64(v.edge_traversals);
        }
        w.u64(self.vehicles.len() as u64);
        w.u64(self.flow_this_tick);
        w.u64(self.last_step_population); // 决策5 豆包 R1：demand_pressure 的 k 来源，
                                          // 权威输入必须入摘要（G3 覆盖），否则不同状态
                                          // 同摘要而降级压力不同=强读档等价假通过风险
        for v in &self.vehicles {
            w.u64(v.id);
            w.u32(v.x);
            w.u32(v.y);
            w.u32(v.goal_x);
            w.u32(v.goal_y);
            w.u64(v.edge_traversals);
        }
        g.record("sim-traffic", "TrafficTick", w.as_bytes());
    }
}

/// 拟合误差报告（3a 独立参照验证口径；全定点‰，权威 crate 禁浮点）。
/// 三区口径（2026-11-19 校准复核）：自由流 k<k_turn/2、转折区 [k_turn/2, 4·k_turn)、
/// 饱和区 k≥4·k_turn（转折区宽度系试点数据 B 导出，须由确认集 C 复测）；
/// k_c 位置偏差仅对三角型（descending_branch_observed）有意义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FitErrorReport {
    /// 自由流区（k < k_turn/2）最大相对误差（‰）
    pub free_flow_max_err_permille: u64,
    /// 转折区（k_turn/2 ≤ k < 4·k_turn）最大相对误差（‰）——只记录不设硬判据
    pub transition_max_err_permille: u64,
    /// 饱和区（k ≥ 4·k_turn）最大相对误差（‰）
    pub saturated_max_err_permille: u64,
    /// 临界密度位置误差（|k_c_拟合 − k_c_观测|，‰）；平台型标 N/A 不判
    pub k_c_err_permille: u64,
    /// 参与评估的留出样本点数
    pub points_evaluated: usize,
    /// 各区样本数（决策4(d)/决策5 共识：空区不得真空 PASS；<2 在判定层标警示）
    pub free_n: usize,
    pub trans_n: usize,
    pub sat_n: usize,
}

/// 判据常量（单一来源，ADR-NEW-25 3a 判据与代码断言必须引用同一常量——
/// 决策4 整改项 (f)：原 simulation 测试断言 250‰ 与判据 100‰ 门禁脱钩）。
/// 复测协议（Set D+第二拓扑+预注册 fixture）后由决策4 复核回填。
pub const FREE_FLOW_CRIT_PERMILLE: u64 = 100;
pub const TRANSITION_CRIT_PERMILLE: u64 = 250;
pub const SATURATED_CRIT_PERMILLE: u64 = 250;
pub const K_C_CRIT_PERMILLE: u64 = 200;

/// 模型流量 q_model(k)（‰）：平台型 min(v_free·k/1000, q_max)；
/// 三角型在下降支 q_max − w·(k−k_c)（负值截 0）。
/// （2026-11-19 决策4 整改：公开供 example/测试复用，删除 mfd3a.rs 内联副本。）
pub fn q_model_permille(fit: &TriangleFit, k_permille: u64) -> u64 {
    let q = fit.v_free_permille as u128 * k_permille as u128 / 1000;
    let q = q.min(fit.q_max_permille as u128) as u64;
    if fit.descending_branch_observed && k_permille > fit.k_c_permille {
        let drop = fit.w_permille as u128 * (k_permille - fit.k_c_permille) as u128 / 1000;
        q.saturating_sub(drop as u64)
    } else {
        q
    }
}

/// 用留出样本（未参与拟合的密度点）评估拟合误差（ADR-NEW-25 3a 独立参照）。
/// 误差=|q_model − q_obs|·1000 / max(q_obs, 1)（‰）；k_c 观测值=留出样本 argmax q 的密度。
/// 分区（审计修复 2026-11-20）：预注册绝对带 ZoneBands（禁由被评参数导出，决策4 3a(a)）。
pub fn evaluate_fit(
    fit: &TriangleFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    // 委托 evaluate_q_fn（X3 泛化核）：分区统计/计数单一实现（防双实现漂移）；
    // 仅 k_c 位置误差为三角型特有。
    let mut r = evaluate_q_fn(|k| q_model_permille(fit, k), holdout, bands);
    let k_obs_peak = holdout
        .iter()
        .max_by_key(|s| s.q_permille)
        .map(|s| s.k_permille)
        .unwrap_or(0);
    r.k_c_err_permille = fit.k_c_permille.abs_diff(k_obs_peak);
    r
}

// ===========================================================================
// 凹形有理饱和族（2026-11-19 决策4 定案默认族；ADR-NEW-25 决策 1）
// q(k) = q_max · k / (k + k_half)；定点 +−×÷ only（禁 exp/log/powf）。
// ===========================================================================

/// 有理饱和拟合参数（定点整数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RationalSaturationFit {
    /// 渐近流量上界 q_max（‰）
    pub q_max_permille: u64,
    /// 半饱和密度 k_half（‰）——q(k_half)=q_max/2；零密度斜率=q_max·1000/k_half。
    pub k_half_permille: u64,
}

impl RationalSaturationFit {
    /// 零密度斜率（‰，q/k 放大 1000 倍）；物理上界=1000‰。
    pub fn zero_density_slope_permille(&self) -> u64 {
        if self.k_half_permille == 0 { return u64::MAX; }
        (self.q_max_permille as u128 * 1000 / self.k_half_permille as u128) as u64
    }
}

/// 定点模型流量 q(k)=q_max·k/(k+k_half)（‰），就近舍入（与 SimScalar 契约同口径）。
pub fn q_rational_permille(fit: &RationalSaturationFit, k_permille: u64) -> u64 {
    if k_permille == 0 { return 0; }
    let num = fit.q_max_permille as u128 * k_permille as u128;
    let den = k_permille as u128 + fit.k_half_permille as u128; // u128 先扩后加（GPT-6 E2 防 u64 环绕）
    ((num + den / 2) / den) as u64
}

/// 辨识（确定性定点）：q_max 与 k_half **联合**整数网格最小化残差平方和
/// （u128 定点；平局取先遇到的最小 (q_max, k_half)）。q_max 候选域=观测最大值
/// 至其 4 倍+16（渐近值在未饱和数据下不可由观测最大值代表——W1 实测教训）；
/// k_half 候选域=1..4·k_max。
/// **越界判失败口径（决策4 条件⑧，审计修复 2026-11-20）**：先做无约束自由估计；
/// 若零密度斜率超 1000‰（每车每 tick 至多一条边），**判定层直接 FAIL**（binding=true
/// 即失败信号）；物理可行域回退解**仅作诊断披露与敏感性参考，不得参与 PASS/FAIL
/// 判定**——旧口径"回退解参与判定"=把约束当估计用，与决策4 条件⑧相反，已废
/// （旧 v_free=min(1000) 截断掩盖同样已废）。
/// 返回 (拟合, 自由估计零密度斜率‰, 约束是否绑定)。
pub fn fit_rational_saturation_detailed(
    samples: &[MfdSample],
) -> (RationalSaturationFit, u64, bool) {
    assert!(!samples.is_empty(), "no samples");
    // q≤k 不变量下沉到拟合器入口（决策5 豆包 R3）：示例外的任何调用方
    // 亦不得喂入 q>k 样本（单位斜率物理界，违者=采样口径缺陷）。
    assert!(
        samples.iter().all(|s| s.q_permille <= s.k_permille),
        "q>k sample violates unit-slope bound (sampling population mismatch)"
    );
    let q_obs_max = samples.iter().map(|s| s.q_permille).max().unwrap().max(1);
    let k_max = samples.iter().map(|s| s.k_permille).max().unwrap().max(1);
    let mut best_free: Option<(u128, u64, u64)> = None;
    let mut best_feasible: Option<(u128, u64, u64)> = None;
    for q_max in q_obs_max..=q_obs_max.saturating_mul(4).saturating_add(16) {
        for k_half in 1..=k_max.saturating_mul(4).max(1) {
            let fit = RationalSaturationFit { q_max_permille: q_max, k_half_permille: k_half };
            let mut sse: u128 = 0;
            for s in samples {
                // 目标=相对误差平方和（‰口径）——审计修复 2026-11-20（豆包 c）：
                // 旧绝对 SSE 与判据的分区相对误差不同构（自由流区权重≈0），
                // 目标与判据同构后自由流点获得应有权重。
                let e = q_rational_permille(&fit, s.k_permille) as i128 - s.q_permille as i128;
                let er = (e * 1000) / (s.q_permille.max(1) as i128);
                sse += (er * er) as u128;
            }
            if best_free.is_none_or(|(b, _, _)| sse < b) {
                best_free = Some((sse, q_max, k_half));
            }
            if k_half >= q_max && best_feasible.is_none_or(|(b, _, _)| sse < b) {
                best_feasible = Some((sse, q_max, k_half));
            }
        }
    }
    let (_, q_free, k_free) = best_free.unwrap();
    let free_slope = (q_free as u128 * 1000 / k_free as u128) as u64;
    if free_slope <= 1000 {
        let fit = RationalSaturationFit { q_max_permille: q_free, k_half_permille: k_free };
        return (fit, free_slope, false);
    }
    let (_, q_f, k_f) = best_feasible.expect("no physically feasible fit (slope<=1000‰)");
    (
        RationalSaturationFit { q_max_permille: q_f, k_half_permille: k_f },
        free_slope,
        true,
    )
}

/// 便捷入口=详细版第 1 返回值（物理可行拟合）。
pub fn fit_rational_saturation(samples: &[MfdSample]) -> RationalSaturationFit {
    fit_rational_saturation_detailed(samples).0
}

/// 广义凹形族（p-norm p=2）：q(k) = q_max·k / sqrt(k² + k0²)。
/// X3 审计整改（豆包清单 d）：多族并行辨识，避免"单族以偏概全"——
/// W3 转 C 的复测只用了有理族一支，豆包实证 p-norm p=2 在同数据上可全过。
/// 定点 +−×÷ + 整数平方根（u128::isqrt），无超越函数（确定性红线保持）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PNormConcaveFit {
    /// 渐近流量上界 q_max（‰）
    pub q_max_permille: u64,
    /// 尺度密度 k0（‰）——q(k0)=q_max/√2；零密度斜率=q_max·1000/k0。
    pub k0_permille: u64,
}

impl PNormConcaveFit {
    /// 零密度斜率（‰）；物理上界=1000‰。
    pub fn zero_density_slope_permille(&self) -> u64 {
        if self.k0_permille == 0 { return u64::MAX; }
        (self.q_max_permille as u128 * 1000 / self.k0_permille as u128) as u64
    }
}

/// 定点模型流量 q(k)=q_max·k/sqrt(k²+k0²)（‰），就近舍入。
pub fn q_pnorm_permille(fit: &PNormConcaveFit, k_permille: u64) -> u64 {
    if k_permille == 0 { return 0; }
    let q_max = fit.q_max_permille as u128;
    let k = k_permille as u128;
    let k0 = fit.k0_permille as u128;
    let den = (k * k + k0 * k0).isqrt().max(1);
    let num = q_max * k;
    ((num + den / 2) / den) as u64
}

/// p-norm 族辨识（目标判据同构：相对误差平方和；越界判失败口径同有理族）。
/// q_max 候选域=观测最大值至 4 倍+16；k0 候选域=1..4·k_max；可行域 k0≥q_max。
pub fn fit_pnorm_concave_detailed(
    samples: &[MfdSample],
) -> (PNormConcaveFit, u64, bool) {
    assert!(!samples.is_empty(), "no samples");
    // q≤k 不变量下沉到拟合器入口（决策5 豆包 R3）：示例外的任何调用方
    // 亦不得喂入 q>k 样本（单位斜率物理界，违者=采样口径缺陷）。
    assert!(
        samples.iter().all(|s| s.q_permille <= s.k_permille),
        "q>k sample violates unit-slope bound (sampling population mismatch)"
    );
    let q_obs_max = samples.iter().map(|s| s.q_permille).max().unwrap().max(1);
    let k_max = samples.iter().map(|s| s.k_permille).max().unwrap().max(1);
    let mut best_free: Option<(u128, u64, u64)> = None;
    let mut best_feasible: Option<(u128, u64, u64)> = None;
    for q_max in q_obs_max..=q_obs_max.saturating_mul(4).saturating_add(16) {
        for k0 in 1..=k_max.saturating_mul(4).max(1) {
            let fit = PNormConcaveFit { q_max_permille: q_max, k0_permille: k0 };
            let mut sse: u128 = 0;
            for s in samples {
                let e = q_pnorm_permille(&fit, s.k_permille) as i128 - s.q_permille as i128;
                let er = (e * 1000) / (s.q_permille.max(1) as i128);
                sse += (er * er) as u128;
            }
            if best_free.is_none_or(|(b, _, _)| sse < b) {
                best_free = Some((sse, q_max, k0));
            }
            if k0 >= q_max && best_feasible.is_none_or(|(b, _, _)| sse < b) {
                best_feasible = Some((sse, q_max, k0));
            }
        }
    }
    let (_, q_free, k_free) = best_free.unwrap();
    let free_slope = (q_free as u128 * 1000 / k_free as u128) as u64;
    if free_slope <= 1000 {
        return (PNormConcaveFit { q_max_permille: q_free, k0_permille: k_free }, free_slope, false);
    }
    let (_, q_f, k_f) = best_feasible.expect("no physically feasible fit (slope<=1000‰)");
    (PNormConcaveFit { q_max_permille: q_f, k0_permille: k_f }, free_slope, true)
}

/// 便捷入口=详细版第 1 返回值。
pub fn fit_pnorm_concave(samples: &[MfdSample]) -> PNormConcaveFit {
    fit_pnorm_concave_detailed(samples).0
}

/// 分区带（审计修复 2026-11-20：判定边界须为**预注册固定 fixture**，禁止由
/// 被评模型参数导出——决策4 3a(a)；自指边界=同参 gaming，豆包实证：自指带
/// 下 FAIL 的数据在绝对带上三区全 PASS）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneBands {
    /// 自由流区上界（k < free_max_k，‰）
    pub free_max_k: u64,
    /// 转折区上界（free_max_k ≤ k < trans_max_k，‰）
    pub trans_max_k: u64,
}

impl ZoneBands {
    /// Set D 主判定带（预注册绝对密度边界，豆包审计复算带）。
    /// 敏感性带（如 free=64/263）由调用方另行构造披露。
    pub const SET_D: ZoneBands = ZoneBands { free_max_k: 100, trans_max_k: 2000 };
}

/// 用留出样本评估拟合（ADR-NEW-25 3a 独立参照；分区=预注册绝对带）。
/// 泛化核：对任意模型 q(k) 评估（X3 审计整改：消除各族评估的双实现，
/// 豆包/MiniMax 报告 q_model 副本漂移教训）。
pub fn evaluate_q_fn(
    q: impl Fn(u64) -> u64,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    assert!(!holdout.is_empty(), "no holdout samples");
    let mut free_max = 0u64;
    let mut trans_max = 0u64;
    let mut sat_max = 0u64;
    let (mut free_n, mut trans_n, mut sat_n) = (0usize, 0usize, 0usize);
    for s in holdout {
        let qm = q(s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000
            / (s.q_permille.max(1) as u128);
        let err = err as u64;
        if s.k_permille < bands.free_max_k {
            free_max = free_max.max(err);
            free_n += 1;
        } else if s.k_permille < bands.trans_max_k {
            trans_max = trans_max.max(err);
            trans_n += 1;
        } else {
            sat_max = sat_max.max(err);
            sat_n += 1;
        }
    }
    FitErrorReport {
        free_flow_max_err_permille: free_max,
        transition_max_err_permille: trans_max,
        saturated_max_err_permille: sat_max,
        k_c_err_permille: 0, // 平台/饱和族无峰值拐点，N/A
        points_evaluated: holdout.len(),
        free_n,
        trans_n,
        sat_n,
    }
}

/// 有理饱和族评估（evaluate_q_fn 委托）。
pub fn evaluate_rational_fit(
    fit: &RationalSaturationFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    evaluate_q_fn(|k| q_rational_permille(fit, k), holdout, bands)
}

/// p-norm 族评估（evaluate_q_fn 委托）。
pub fn evaluate_pnorm_fit(
    fit: &PNormConcaveFit,
    holdout: &[MfdSample],
    bands: &ZoneBands,
) -> FitErrorReport {
    evaluate_q_fn(|k| q_pnorm_permille(fit, k), holdout, bands)
}

/// (k, q) 采样点：k=活跃车辆/边（放大 1000 倍定点），q=边遍历/边每 tick（放大 1000 倍）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MfdSample {
    pub k_permille: u64,
    pub q_permille: u64,
}

/// 三角基本图拟合（定点；候选模型形式，ADR-NEW-25 决策 1）。
/// 2026-11-19 投票修复：自由流斜率改 k≤k_c 点过原点定点最小二乘（原三点估计
/// v_free=q_max/k_first 高估约 31 倍）；拥堵斜率仅在观测到下降支时辨识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriangleFit {
    /// 自由流斜率（q/k，放大 1000 倍；物理上界=1000，即每车每 tick 一条边）。
    pub v_free_permille: u64,
    /// 临界密度（峰值处 k，放大 1000 倍）。
    pub k_c_permille: u64,
    /// 峰值流量（放大 1000 倍）。
    pub q_max_permille: u64,
    /// 拥堵斜率绝对值（放大 1000 倍；未观测到下降支时为 0=不可辨识）。
    pub w_permille: u64,
    /// 是否观测到拥堵下降支（q 在 k>k_c 处低于 q_max）。false=数据呈单调饱和
    /// 平台型，三角的 -w 分支不可辨识，模型应按平台型（单调饱和凹曲线）解读。
    pub descending_branch_observed: bool,
}

/// 从采样序列拟合基本图（确定性：输入序即计算序）。
/// 采样须按 k 升序传入（调用方排序，键相同按采样序稳定）。
pub fn fit_triangle(samples: &[MfdSample]) -> TriangleFit {
    assert!(!samples.is_empty(), "no samples");
    // 注（决策5）：q≤k 采样不变量的真实数据保障在两个 detailed 拟合器入口断言
    // 与 example 采集点 fail-fast；fit_triangle 为遗留估计器，其 clamp 语义
    // （v_free.min(1000)）需要越界合成数据才能测试，故此处不断言。
    let mut peak = samples[0];
    for s in samples {
        if s.q_permille > peak.q_permille {
            peak = *s;
        }
    }
    let k_c = peak.k_permille.max(1);
    let q_max = peak.q_permille;
    // 自由流斜率=相邻采样点最大割线斜率 max(Δq/Δk)（k 升序输入）。
    // 语义=零密度导数的可观测上界；对三角数据与平台型数据均正确。
    // （2026-11-19 二次修正：过原点全点最小二乘在单调饱和数据下被高 k 点拉低。）
    let mut v_free = if samples[0].k_permille > 0 {
        (samples[0].q_permille as u128 * 1000 / samples[0].k_permille as u128) as u64
    } else {
        0
    };
    for pair in samples.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if b.k_permille > a.k_permille {
            let slope = ((b.q_permille.saturating_sub(a.q_permille)) as u128 * 1000
                / (b.k_permille - a.k_permille) as u128) as u64;
            if slope > v_free {
                v_free = slope;
            }
        }
    }
    // 物理上界截断：本微观模型每车每 tick 至多一条边 → v_free ≤ 1000‰；
    // 离散化下割线斜率可短暂越界（如 Δq/Δk≈1010‰），按契约截 0~1000。
    let v_free = v_free.min(1000);
    // 拥堵斜率：仅当 k>k_c 处存在 q<q_max 的下降支样本才可辨识；
    // 对 k>k_c 点拟合 q ≈ q_max - w·(k-k_c)：w = Σ(Δk·Δq)/Σ(Δk²)（负值截 0）。
    let descending = samples
        .iter()
        .any(|s| s.k_permille > k_c && s.q_permille < q_max);
    let mut w = 0u64;
    if descending {
        let (mut n2, mut d2) = (0i128, 0i128);
        for s in samples {
            if s.k_permille > k_c {
                let dk = (s.k_permille - k_c) as i128;
                let dq = q_max as i128 - s.q_permille as i128;
                n2 += dk * dq;
                d2 += dk * dk;
            }
        }
        if d2 > 0 && n2 > 0 {
            w = (n2 * 1000 / d2) as u64;
        }
    }
    TriangleFit {
        v_free_permille: v_free,
        k_c_permille: k_c,
        q_max_permille: q_max,
        w_permille: w,
        descending_branch_observed: descending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_run(densities: &[u64]) -> (GlobalDigest, Vec<MfdSample>) {
        let mut g = GlobalDigest::new();
        let mut samples = Vec::new();
        for &inj in densities {
            let mut gt = GridTraffic::new(8, 8, 1, 0x00C0_FFEE_1234_5678);
            for t in 0..260u64 {
                // 密度阶梯=持续注入；200 tick 后进入稳态采样
                gt.inject(inj);
                let _ = t;
                gt.step();
                gt.fold_digest(&mut g);
            }
            samples.push(MfdSample {
                k_permille: gt.active_count() * 1000 / gt.edge_count(),
                q_permille: gt.flow_this_tick * 1000 / gt.edge_count(),
            });
        }
        (g, samples)
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
    }

    #[test]
    fn tbench_same_seed_two_runs_agree() {
        let (g1, s1) = small_run(&[1, 2, 4]);
        let (g2, s2) = small_run(&[1, 2, 4]);
        assert_eq!(g1.finalize(), g2.finalize());
        assert_eq!(s1, s2);
    }

    #[test]
    fn mfd_shape_basic() {
        let (_, samples) = small_run(&[1, 2, 4, 8, 16]);
        // q 非负、峰值存在、k 单调可排
        assert!(samples.iter().any(|s| s.q_permille > 0));
        let peak = samples.iter().max_by_key(|s| s.q_permille).unwrap();
        assert!(peak.q_permille > 0);
    }

    #[test]
    fn triangle_fit_is_deterministic() {
        let (_, mut samples) = small_run(&[1, 2, 4, 8]);
        samples.sort_by_key(|s| s.k_permille);
        let f1 = fit_triangle(&samples);
        let f2 = fit_triangle(&samples);
        assert_eq!(f1, f2);
        assert!(f1.q_max_permille > 0);
        assert!(f1.k_c_permille > 0);
    }

    // —— 2026-11-19 投票修复回归 ——

    #[test]
    fn free_flow_slope_is_physically_bounded() {
        // 物理上界：每车每 tick 至多一条边 → v_free ≤ 1000‰。
        // 旧三点估计（q_max/k_first）高估约 31 倍，此测试为回归护栏。
        let (_, mut samples) = small_run(&[1, 2, 4, 8, 16]);
        samples.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&samples);
        assert!(f.v_free_permille > 0, "v_free = {}", f.v_free_permille);
        assert!(
            f.v_free_permille <= 1000,
            "v_free = {}‰ exceeds physical bound 1000‰",
            f.v_free_permille
        );
    }

    #[test]
    fn monotone_saturation_reports_no_descending_branch() {
        // 点队列模型无 spillback，(k,q) 单调饱和 → 三角 -w 分支不可辨识，
        // 拟合须显式报告 descending_branch_observed=false（平台型解读）。
        let (_, mut samples) = small_run(&[2, 4, 8, 16, 32]);
        samples.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&samples);
        if samples.iter().filter(|s| s.k_permille > f.k_c_permille).all(|s| s.q_permille >= f.q_max_permille)
        {
            assert!(!f.descending_branch_observed);
            assert_eq!(f.w_permille, 0);
        }
    }

    #[test]
    fn synthetic_triangle_is_recovered() {
        // 人工合成精确三角：v_free=1000‰、k_c=100‰、q_max=100‰、w=500‰。
        let samples = vec![
            MfdSample { k_permille: 10, q_permille: 10 },
            MfdSample { k_permille: 50, q_permille: 50 },
            MfdSample { k_permille: 100, q_permille: 100 },
            MfdSample { k_permille: 200, q_permille: 50 },
            MfdSample { k_permille: 300, q_permille: 0 },
        ];
        let f = fit_triangle(&samples);
        assert_eq!(f.v_free_permille, 1000);
        assert_eq!(f.k_c_permille, 100);
        assert_eq!(f.q_max_permille, 100);
        assert!(f.descending_branch_observed);
        assert_eq!(f.w_permille, 500);
    }

    #[test]
    fn explicit_id_injection_is_order_independent() {
        // 2026-11-19 投票修复：同一 ID 集合以不同物理顺序注入，权威状态（摘要）必须一致。
        let run = |order: &[u64]| {
            let mut gt = GridTraffic::new(8, 8, 1, 0x00C0_FFEE_1234_5678);
            for &id in order {
                gt.inject_with_id(id);
            }
            let mut g = GlobalDigest::new();
            gt.fold_digest(&mut g);
            for _ in 0..10u64 {
                gt.step();
                gt.fold_digest(&mut g);
            }
            g.finalize()
        };
        let d1 = run(&[3, 1, 2, 5, 4]);
        let d2 = run(&[1, 2, 3, 4, 5]);
        let d3 = run(&[5, 4, 3, 2, 1]);
        assert_eq!(d1, d2, "injection order must not matter");
        assert_eq!(d2, d3, "injection order must not matter");
    }

    #[test]
    fn fold_digest_covers_goal_and_next_id() {
        // 强读档等价护栏：goal 与 next_id 属权威状态，摘要必须敏感。
        let base = |mut gt: GridTraffic| {
            for _ in 0..5u64 {
                gt.inject(3);
                gt.step();
            }
            let mut g = GlobalDigest::new();
            gt.fold_digest(&mut g);
            g.finalize()
        };
        let d1 = base(GridTraffic::new(8, 8, 1, 7));
        let d2 = base(GridTraffic::new(8, 8, 1, 7));
        assert_eq!(d1, d2);

        // next_id 敏感
        let mut gt = GridTraffic::new(8, 8, 1, 7);
        for _ in 0..5u64 {
            gt.inject(3);
            gt.step();
        }
        gt.next_id += 1;
        let mut g = GlobalDigest::new();
        gt.fold_digest(&mut g);
        assert_ne!(d1, g.finalize(), "next_id must be covered");

        // goal 敏感
        let mut gt = GridTraffic::new(8, 8, 1, 7);
        for _ in 0..5u64 {
            gt.inject(3);
            gt.step();
        }
        if let Some(v) = gt.vehicles.first_mut() {
            v.goal_x = v.goal_x.wrapping_add(1);
        }
        let mut g = GlobalDigest::new();
        gt.fold_digest(&mut g);
        assert_ne!(d1, g.finalize(), "vehicle goal must be covered");
    }

    #[test]
    fn synthetic_holdout_meets_adr25_criteria() {
        // 精确三角（v_free=1000‰, k_c=100‰, q_max=100‰, w=500‰）：
        // 拟合用 (10,100,300) 三点，留出 (50,200) 两点——独立于拟合集评估。
        let fit_in = vec![
            MfdSample { k_permille: 10, q_permille: 10 },
            MfdSample { k_permille: 100, q_permille: 100 },
            MfdSample { k_permille: 300, q_permille: 0 },
        ];
        let holdout = vec![
            MfdSample { k_permille: 50, q_permille: 50 },
            MfdSample { k_permille: 200, q_permille: 50 },
        ];
        let f = fit_triangle(&fit_in);
        assert!(f.descending_branch_observed);
        assert_eq!(f.v_free_permille, 1000);
        assert_eq!(f.w_permille, 500);
        let r = evaluate_fit(&f, &holdout, &ZoneBands::SET_D);
        // ADR-NEW-25 3a 判据（原 10/20/25% 三区重分）：自由流 ≤100‰、饱和区 ≤250‰、
        // k_c ≤200‰（三角）；转折区只记录不设硬判据。
        assert!(r.free_flow_max_err_permille <= 100, "{}", r.free_flow_max_err_permille);
        assert!(r.k_c_err_permille <= 200, "{}", r.k_c_err_permille);
        assert!(r.saturated_max_err_permille <= 250, "{}", r.saturated_max_err_permille);
    }

    #[test]
    #[ignore = "债务 #31 审计退回（2026-11-20）：保持 ignore，直至口径修复后 Set D 复测通过并经负责人追认（FAIL 翻转则按重投裁决处置）"]
    fn simulation_holdout_free_flow_is_accurate() {
        // 两次独立小场景（同种子不同密度阶梯）：A 拟合、B 留出——独立密度点评估。
        let (_, mut a) = small_run(&[1, 2, 4, 8]);
        let (_, b) = small_run(&[3, 6, 12]);
        a.sort_by_key(|s| s.k_permille);
        let f = fit_triangle(&a);
        let r = evaluate_fit(&f, &b, &ZoneBands::SET_D);
        assert!(r.points_evaluated >= 2);
        // 门禁对齐（决策4 整改）：断言引用判据常量（100‰），不再用 250‰ 脱钩阈值。
        // 当前点队列模型自由流实测 190‰ FAIL——债务 #31（凹形扩展）Set D 复测通过前
        // 本测试标 ignore，避免"FAIL 仍绿"掩盖真实状态（豆包/GPT-6 指出）。
        assert!(
            r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE,
            "free-flow err {}‰",
            r.free_flow_max_err_permille
        );
    }

    // ---- 凹形有理饱和族（决策4 W1）----

    fn rational_samples(q_max: u64, k_half: u64, ks: &[u64]) -> Vec<MfdSample> {
        let fit = RationalSaturationFit { q_max_permille: q_max, k_half_permille: k_half };
        ks.iter()
            .map(|&k| MfdSample { k_permille: k, q_permille: q_rational_permille(&fit, k) })
            .collect()
    }

    #[test]
    fn synthetic_rational_is_recovered() {
        // 人工合成精确有理饱和图（q_max=200‰、k_half=150‰）→辨识恢复。
        let ks: Vec<u64> = (1..=40).map(|i| i * 25).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks));
        assert_eq!(fit.q_max_permille, 200);
        assert_eq!(fit.k_half_permille, 250);
    }

    #[test]
    fn rational_identification_is_deterministic() {
        let ks: Vec<u64> = (1..=30).map(|i| i * 17).collect();
        let a = fit_rational_saturation(&rational_samples(200, 250, &ks));
        let b = fit_rational_saturation(&rational_samples(200, 250, &ks));
        assert_eq!(a, b);
    }

    #[test]
    fn rational_zero_density_slope_is_bounded() {
        let ks: Vec<u64> = (1..=30).map(|i| i * 33).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks));
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn rational_bound_violation_is_disclosed_and_constrained() {
        // 越界判失败口径（决策4）：自由估计越界→披露幅度+物理可行解回退，
        // 不 panic 不截断。合成 (500,50) 自由斜率 10000‰ 必越界。
        // 样本取 k≥500 保证 q≤k（采样不变量断言下合法），模型越界仍成立。
        let ks: Vec<u64> = vec![500, 1000, 2000];
        let (fit, free_slope, binding) =
            fit_rational_saturation_detailed(&rational_samples(500, 50, &ks));
        assert!(binding, "bound must bind");
        assert_eq!(free_slope, 10_000, "violation magnitude must be disclosed");
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn rational_holdout_meets_criteria_on_synthetic() {
        // 合成数据留出评估：三区判据应全过（常量单一来源）。
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let fit = fit_rational_saturation(&rational_samples(200, 250, &ks));
        let holdout: Vec<u64> = (1..=12).map(|i| i * 40 + 13).collect();
        let r = evaluate_rational_fit(&fit, &rational_samples(200, 250, &holdout), &ZoneBands::SET_D);
        assert!(r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE, "{}", r.free_flow_max_err_permille);
        assert!(r.transition_max_err_permille <= TRANSITION_CRIT_PERMILLE, "{}", r.transition_max_err_permille);
        assert!(r.saturated_max_err_permille <= SATURATED_CRIT_PERMILLE, "{}", r.saturated_max_err_permille);
    }

    // ===== X3：p-norm p=2 族（豆包清单 d：多族并行辨识） =====

    fn pnorm_samples(q_max: u64, k0: u64, ks: &[u64]) -> Vec<MfdSample> {
        let fit = PNormConcaveFit { q_max_permille: q_max, k0_permille: k0 };
        ks.iter()
            .map(|&k| MfdSample { k_permille: k, q_permille: q_pnorm_permille(&fit, k) })
            .collect()
    }

    #[test]
    fn synthetic_pnorm_is_recovered() {
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let (fit, free_slope, binding) = fit_pnorm_concave_detailed(&pnorm_samples(200, 250, &ks));
        assert_eq!(fit, PNormConcaveFit { q_max_permille: 200, k0_permille: 250 });
        assert_eq!(free_slope, 800);
        assert!(!binding);
    }

    #[test]
    fn pnorm_identification_is_deterministic() {
        let ks: Vec<u64> = (1..=16).map(|i| i * 40).collect();
        let s = pnorm_samples(300, 400, &ks);
        assert_eq!(fit_pnorm_concave_detailed(&s), fit_pnorm_concave_detailed(&s));
    }

    #[test]
    fn pnorm_bound_violation_is_disclosed_and_constrained() {
        // 自由估计斜率超界（500/50=10000‰）→binding=true 即失败信号；
        // 可行解回退仅诊断，zero_density_slope≤1000‰。样本 k≥500 保证 q≤k。
        let ks: Vec<u64> = vec![500, 1000, 2000];
        let (fit, free_slope, binding) = fit_pnorm_concave_detailed(&pnorm_samples(500, 50, &ks));
        assert!(binding);
        assert!(free_slope > 1000);
        assert!(fit.zero_density_slope_permille() <= 1000);
    }

    #[test]
    fn pnorm_holdout_meets_criteria_on_synthetic() {
        let ks: Vec<u64> = (1..=24).map(|i| i * 30).collect();
        let fit = fit_pnorm_concave(&pnorm_samples(200, 250, &ks));
        let holdout: Vec<u64> = (1..=12).map(|i| i * 40 + 13).collect();
        let r = evaluate_pnorm_fit(&fit, &pnorm_samples(200, 250, &holdout), &ZoneBands::SET_D);
        assert!(r.free_flow_max_err_permille <= FREE_FLOW_CRIT_PERMILLE, "{}", r.free_flow_max_err_permille);
        assert!(r.transition_max_err_permille <= TRANSITION_CRIT_PERMILLE, "{}", r.transition_max_err_permille);
        assert!(r.saturated_max_err_permille <= SATURATED_CRIT_PERMILLE, "{}", r.saturated_max_err_permille);
    }

    #[test]
    fn pnorm_isqrt_rounding_golden() {
        // isqrt 舍入黄金（决策5 共识收口条件）：手算定点期望值钉死语义——
        // q = round(q_max·k·1000 / isqrt((k²+k0²)·10⁶))，就近舍入 ((num+den/2)/den)。
        let f = PNormConcaveFit { q_max_permille: 200, k0_permille: 250 };
        // 注意 isqrt floor 偏差披露（决策5）：den 先取整再除，使结果较真值
        // 有 +0~+1‰ 的系统性上偏（如 k=250 真值 141.42→定点 142）。
        assert_eq!(q_pnorm_permille(&f, 0), 0);
        // k=250：真值 141.42‰，定点就近→142（isqrt floor 偏差）
        assert_eq!(q_pnorm_permille(&f, 250), 142);
        // k=1000：真值 194.03‰ → 194
        assert_eq!(q_pnorm_permille(&f, 1000), 194);
        // k=2500：真值 198.03‰，定点→199（isqrt floor 偏差）
        assert_eq!(q_pnorm_permille(&f, 2500), 199);
    }
}


---

## FILE: decision-log.md

# CitySim-rs 决策日志（decision-log）

本文件记录 ADR 的状态与变更历史。编号为整合后的暂定索引，正式入仓时按仓库 ADR 序列重新编号并保留映射。

## V1.9.3 新增决策（源自五项修订提案，2026-11-18 并入主文档）

| ADR | 主题 | 当前状态 | 裁决 | 待决事项 |
| --- | --- | --- | --- | --- |
| ADR-NEW-14 | 预算与降级必须确定性 | Accepted（2026-11-19，附条件） | 有条件采纳（2026-11-19 正式裁决，五家共识） | 各系统工作量计量单位与每 tick 上限阈值；声明载体与 L1/L2/L3 映射（债务 #16）；降级触发/恢复路径实测一致性入 docs/benchmarks/；R2/R3 分层后门禁实测耗时；预算滚动状态生命周期（债务 #20） |
| ADR-NEW-15 | 确定性边界定义 | Accepted（2026-11-19） | 采纳（2026-11-19 正式裁决） | clippy 分层实测通过（docs/benchmarks/clippy-layering-REPORT.md）；条 5 缺省序已补（全局系统提交按 (phase_id, system_id) 静态序）；跨平台一致性（R4）；Rapier ADR 豁免机制（若承担权威物理） |
| ADR-NEW-16 | 存档最小集合扩充 | Accepted（2026-11-19，附条件） | 有条件采纳（2026-11-19 正式裁决） | 在途任务恢复策略"保存并恢复"已定案；EC-03 表增补已落实（三类 ID 高水位+让步状态，2026-11-19）；永久兼容边界；存档序列化格式（债务 #11，随 ADR-NEW-19）；回档验证入 EC-07 正式测试集 |
| ADR-NEW-17 | 路网事务的过期结果策略 | Accepted（2026-11-19，联动组定案，实施前置） | 有条件采纳（联动组，2026-11-19 正式裁决） | 实施前置=联动组级验收 G1-G5+两阶段窄判测试覆盖；条 8 与 NEW-23 P1-04 语义已调和（2026-11-19 同步注）；相交粒度/重算上限/兜底比选/行走删路取数见 ADR-NEW-23 |
| ADR-NEW-18 | 仿真参数基线与数值契约补充 | Accepted（2026-11-19，附条件） | 有条件采纳（2026-11-19 正式裁决） | 七参数已定案（CS2 参考，标注"设计参考非官方实测"）；舍入规则 Rust 专项验证（截断对比黄金）为转 Verified 前置；证据声明已勘误（Rust 实测已入仓）；Step 2 已用定案参数重跑 |

裁决口径（见主文档第 7 章）：采纳 / 有条件采纳 / 拒绝 / 延后，每项裁决须记录理由和验证条件。

## 2026-11-18 评审后新增 ADR 草案（源自设计审计）

来源：开工前设计审计（25 条发现，其中阻塞级 3 条）。审计结论：写代码前先补三件事——GlobalDigest ADR、请求 ID/在途任务口径定案、未登记债务入账。前两件立项如下，第三件见下一节。

| ADR | 主题 | 当前状态 | 裁决 | 待决事项 |
| --- | --- | --- | --- | --- |
| ADR-NEW-19 | GlobalDigest 摘要算法与规范化序列化 | Accepted（2026-11-19，附条件） | 有条件采纳（2026-11-19 正式裁决） | 最优先实施项（债务 #11/#20/#23 上游）：编码器实施时补黄金测试向量、digest_version bump 规则随编码器；GlobalDigest 正式测试尚不存在（驳回第 5 家"已验证"断言），实施后以 R2 oracle 验证 |
| ADR-NEW-20 | 请求/事务 ID 分配与在途寻路任务口径 | Accepted（2026-11-19，联动组定案，实施前置） | 有条件采纳（联动组；机制 B 为默认定案候选，A/B/C 以 G5 比选后锁定——依评审 F 修正） | 机制 B 三硬约束（放大幅 g 与上限 L≥|V| 常量化+解析上界 k≤⌈log_g(|V|/b)⌉+1；放大=总额不变确定性配额再分配；三计数分离）；请求/事务 ID 单一分配点（债务 #24）；实施前置=G1-G5+压测清单（200×200 极端长路径 A/B/C 对比） |
| ADR-NEW-21 | SimScalar 数值格式定案（Spike A 数值设计决策） | Accepted（2026-11-19） | 采纳（2026-11-19 正式裁决，无条件） | Q24.40 整数位勘误（±2^23≈8.39e6 m，结论不变）；转 Verified 外围条件=债务 #22 工具链收敛、正式基准替代冒烟、dt/漂移黄金值入 R4 门禁 |
| ADR-NEW-22 | 系统级 RNG 协议（Spike B 协议设计决策） | Accepted（2026-11-19，附条件） | 有条件采纳（2026-11-19 正式裁决） | namespace 注册表+注册期 64 位碰撞校验（债务 #25，已补注）；统计质量正式测试（TestU01/PractRand 级）R4 前；协议版本 bump 随 EC-03；跨平台黄金值复验（R4） |
| ADR-NEW-23 | 路网事务过期结果细则与行走删路行为（Spike C 实测定案建议） | Accepted（2026-11-19，联动组定案，实施前置） | 有条件采纳（联动组，2026-11-19 正式裁决） | 实施前置=两阶段（AABB+边 ID 窄判）与代价默认不触发的测试覆盖（6 场景矩阵：误判率/重算率/阻塞率/平均与最大等待 tick）+组级验收；"影响"节矛盾句已勘误；兜底候选启用前定义"部分路径可接受性"（债务 #18）；并发撤销与事务日志回滚点随 ADR-NEW-19 编码器 |
| ADR-NEW-24 | 路网 CSR 与确定性 A* 细则（Stage 2A 实施前置定案） | Accepted（2026-11-19，附条件） | 有条件采纳（第二轮外部评审 8:0） | 条件绑定债务 #27（机制 B→A 切换阈值）、#28（事务重试 tx_id）、#29（全局每 tick 寻路配额）、#30（A* 细则补遗：h 一致性不变量/L=|V|×reopen/前驱裁决/CSR u32 边界）、#8（Edge 语义入数据字典）；QWEN4-MAX 两条代码缺陷已修复（2026-11-19，含 2 条回归测试）；条件截止点=Stage 2B（并发队列+R3/R4+让步强读档专项） |
| ADR-NEW-25 | MFD 交通模型与验证方案 | Accepted（附条件）（2026-11-19）；**2026-11-20 复判：转 C 不成立（待投票确认）** | 有条件采纳（决策2 投票 10/10） | 模型形态按辨识二选一（平台型默认/三角条件启用）、拟合=最大割线+下降支 LSQ；权威性三层拆分；3a 独立参照、3c 生产代码路径=转 Verified 条件（未清不得开工）；**Set D 复测（X1-X4 整改后，证据 commit ce4a705）判定翻转：pnorm-p2 物理可行两拓扑三区全 PASS→原转 C 终局回滚（待投票确认三项）；MFD→降级用途仍冻结** |
| ADR-NEW-26 | 需求压力降级信号通道（MFD→BudgetRegistry） | Proposed（2026-11-19） | 待评审 | demand_pressure_bp 确定性定点→note_demand_pressure；effective=max(工作量,需求)；需求单独触发封顶 L1（债务 #31 限制）；状态入档入摘要；耦合线程无关测试 3 路一致 |

## 未登记决策债务登记（2026-11-18 设计审计入账）

审计发现此前两本账（ADR 待决事项表与首轮裁决表）均未记录以下待决事项；按审计建议扩表入账，逐项给出截止点。

| # | 待决事项 | 关联位置 | 严重级 | 截止点 / 处置 |
| --- | --- | --- | --- | --- |
| 1 | GlobalDigest 哈希/版本/规范化序列化/纳入范围/计算时机 | EC-05、EC-03、EC-11 | 阻塞 | 已立项 ADR-NEW-19，Stage 0 前裁决 |
| 2 | 在途 A* 中间态 × 预算计量 × 强读档等价三方互锁 | ADR-NEW-16、EC-11 | 阻塞 | 已立项 ADR-NEW-20（单 tick 原子搜索） |
| 3 | 请求 ID / 事务 ID 确定性分配器 | EC-03、2.8 | 阻塞 | 已立项 ADR-NEW-20 |
| 4 | RNG 算法 / 派生键编码 / 协议 bump 策略 | EC-02 | 重要 | 已立项 ADR-NEW-22（Spike B 已完成 2026-11-18，双模式+SplitMix64+版本化），待评审裁决 |
| 5 | SimError/错误码/诊断事件 schema；SimErrorContext.entity_id 与 2.3 解耦冲突 | EC-08 | 重要 | Stage 0 |
| 6 | 坐标系 / 单位 / 角度表示 | 2.4、2.10 | 重要 | Stage 0（与 2.10 定案同期） |
| 7 | Agent 状态机与"受阻状态"定义 | EC-01、ADR-NEW-17 条文 10 | 重要 | Stage 1 前 |
| 8 | 路网数据字典；CSR 定案表述矛盾（Accepted vs 待性能测试） | 2.1、2.5 | 重要 | Stage 2A 前（2026-11-19 更新：CSR 表述矛盾已收口=ADR-NEW-24 决策 1；剩余=Edge 有向语义三件事入数据字典——方向性/平行边策略/代价下限不变量，第二轮评审 8/8 要求，Stage 2B 前） |
| 9 | 影响区域表示法（矩形 / 边集 / 瓦片）与请求/结果字段 | EC-01 | 重要 | 已立项 ADR-NEW-23（Spike C 已完成 2026-11-18：轴对齐包围盒 1 mm 网格 + 并集聚合粒度），待评审裁决 |
| 10 | 回放输入 / 模拟配置 schema | 2.7、2.9 条 8、EC-11 | 重要 | Stage 1 前 |
| 11 | 存档序列化格式（与摘要 canonical 编码共用编码器） | EC-03 | 重要 | Stage 1 前，随 ADR-NEW-19 编码器定案；canonical 编码方向已由 save-restore-spike 验证（小端 + u64 长度前缀 + 尾字节拒绝，Step 5） |
| 12 | Schema / 确定性协议 / RNG 协议版本号规则与 bump 策略 | EC-03 | 一般 | Stage 1 前 |
| 13 | 2.8"需要明确"五项（快照机制/结果排序/冲突处理/失败任务/Tick 边界） | 2.8 | 重要 | Stage 1 前 |
| 14 | LOD 三级命名/切换条件/更新频率；LOD 无 Stage 归属 | 2.7、ADR-NEW-14 | 重要 | 排入第五章阶段计划，Stage 2C 前 |
| 15 | crate 权威/非权威分类清单 | 2.2、EC-12 | 重要 | clippy 分层实测已完成（2026-11-18）；清单落点=各权威 crate 包根 clippy.toml + CI ec12-lint 矩阵成员列表（.github/workflows/ci.yml），待正式权威 crate 建库时随 2.2 目录结构定案 |
| 16 | 预算声明载体与 L1/L2/L3 按系统映射 | 2.7、EC-04 | 一般 | **C2 已处置（2026-11-19）**：载体=BudgetRegistry 代码注册表（ADR-NEW-14"C2 细化"节），逐系统计量单位/上限候选表+L1/L2/L3 映射已立；阈值数值（Q=65536、70/90/100、恢复 50%、事务 1024）**已定案**（2026-11-19 决策3 投票 1a/1b/1c/1d；残余细化=事务工作量加权与拓扑/代价上限区分，投票 1d 调整项随实现） |
| 17 | A* 确定性细则（平局打破/邻居展开序/启发式舍入）；Stage 2A 退出判据 | 2.4、第 5 章 | 重要 | Stage 2A 前 |
| 18 | EC-01 取消语义 / 冲突合并 / 验证判据 | EC-01 | 一般 | Spike C |
| 19 | Stage 1 任务清单与原始路线图缺失（须重建入仓） | 第 5/8 章 | 重要 | **已关闭（2026-11-19）**：任务清单重建入仓 docs/architecture/Stage-1-Task-List.md（T1~T9+退出条件 6 条+执行顺序）；原始路线图仍缺失，后续阶段清单随各阶段开工时按同口径重建 |
| 20 | 预算滚动状态生命周期（若每 tick 重置则 tick 边界存档无意义） | EC-03、2.7 | 一般 | ADR-NEW-14 修订定案 |
| 21 | 安全债：ruzstd 0.8.3 高危 RUSTSEC-2026-0344（解压炸弹）基线内不可修复（bevy_image 0.18.1 钉 ^0.8）；paste 1.0.15 / ttf-parser 0.25.1 unmaintained 无修复 | 2.1（安全检查） | 重要 | 随 Bevy 升级 ADR 消化（ruzstd ≥0.9.1 列为升级动机之一）；评估=纹理/资产解码路径属非权威侧（2.9），Stage 0 不加载不可信资产；Step 8 实测见 docs/benchmarks/dep-verify-REPORT.md |
| 22 | Rust 基线 1.89（须 Cargo.lock 锁 ordered-float ≤5.4.0）vs workspace 实测 1.100.0 收敛取舍 | 2.1、rust-toolchain.toml | 一般 | **已关闭（2026-11-19 决策投票 6:1 定案 B）**：文档基线=开发工具链=CI 主工具链=1.100.0；1.89=Bevy 0.18.1 MSRV 参考，msrv-check 保留为门禁；ordered-float 钉版维持至 MSRV 调整复核 |
| 23 | **外部评审 P0-01：单 tick 原子 A* 前向进展/饥饿**——单次搜索规模长期大于每 tick 预算时"中止-从头重试"可能永久完不成；双计数器（budget_yield_count / stale_recompute_count）分离与预算让步机制（A 跨 tick 续算 / B 确定性预算递增 / C 规模上限受控失败）待定案 | ADR-NEW-20、2.7、EC-01、Spike C | 重要 | ADR-NEW-20"预算让步与前向进展"节已给候选比选（默认 B）与验收 6 条；随 ADR-NEW-17/20/23 联动决策组一并裁决；Stage 1 寻路实现前必须定案（2026-11-19 更新：机制 B 已正式定案，含三硬约束与解析上界，待组级验收 G1-G5+压测清单实测后实施） |

| 24 | 请求/事务 ID 分配点唯一性（tick 内固定相位、单一分配点按 (agent_id, local_sequence) 取号、禁止并行阶段分配） | ADR-NEW-20、EC-03 | 重要 | Stage 1 寻路实现前定案（2026-11-19 正式裁决新增，评审 C） |
| 25 | namespace 注册表与 64 位哈希碰撞校验（注册期检测、发布后不得复用或变更语义） | ADR-NEW-22、EC-02 | 一般 | RNG 协议实施前（2026-11-19 正式裁决新增，评审 C/B） |
| 26 | CSR 数据结构定案 + A* 确定性细则（平局打破/邻居展开序/启发式舍入）——GlobalDigest 漂移风险 | 2.1、2.4、Stage 2A（扩债务 #8/#17） | 重要 | Stage 2A 前（2026-11-19 正式裁决新增，评审 D） |
| 27 | 机制 B→A 切换阈值定案（dup_rate 结构性基线 0.53 是 g=2 级数水平，单请求重复率上界→2/3；切换触发候选=最坏单 tick 耗时 SLO 或 dup_rate 超阈；数值需多请求并发/非均匀路网数据） | ADR-NEW-24 决策 3/4、ADR-NEW-20 | 重要 | **保持候选，不得定案**（2026-11-20 债务闭合审计共识：4096/8 为单种子三场景经验候选，过拟合+两阈值共线；定案需多 seed/异型场景/代价侧实测+f(b,g,L) 表达）；基线已实测 docs/benchmarks/G5-threshold-baseline.txt（V4） |
| 28 | 寻路请求取消/冲突重试的 tx_id 语义（沿用原 ID 按全序重排 vs 新 ID 显式链接；重试上限衔接受阻状态） | ADR-NEW-24 取消/冲突、ADR-NEW-17、ADR-NEW-20 | 一般 | **待实装**（2026-11-20 审计更正：语义已定案（沿用原 ID 全序重排+上限 3 转受阻），但 crates 无 retry 实现，原"定案实装"表述不实；实装含 retry_count/上限/受阻终态/测试） |
| 29 | 全局每 tick 寻路配额（B_k≤min(b·2^k,L,剩余配额)、多请求共享配额上界）——L=|V| 使单请求最坏展开=|V|，与 ADR-NEW-14 条 3 每 tick 工作量上限的关系待对齐 | ADR-NEW-24 决策 3、ADR-NEW-14、2.7 | 重要 | Stage 2B 前（2026-11-19 第二轮评审新增，Claude/QWEN） |
| 30 | A* 细则补遗：h 一致性不变量（h(u)≤cost(u,v)+h(v)，动态代价更新后仍须成立）、L=|V|×reopen 上界重证（或一致性免 reopen）、open 集 g 相等先发现保留明文、CSR u32 容量边界（>u32::MAX 确定性失败）、node_id 整表重建前后稳定 | ADR-NEW-24 决策 2 | 重要 | Stage 2B 前（2026-11-19 第二轮评审新增，CHATGPT/Claude/QWEN） |
| 31 | MFD 模型形式扩展（凹形自由流分支） | ADR-NEW-25 决策 1、3a | 重要 | **2026-11-20 复判：转 C 不成立（待投票确认）**：X1-X4 整改后 Set D 复测（证据 commit ce4a705）判定翻转——口径修复后两拓扑三区全 PASS（40×40：83/118/56‰；64×64：80/112/89‰），原自由流 FAIL 系 q>k 测量 bug 误差地板（豆包复算证实）；rational 族自由估计斜率 1095/1092‰ 越界=族不适配（条件⑧淘汰）；**pnorm-p2 族物理可行且两拓扑三区全 PASS→"凹形不可救"不成立**。按豆包 A2 预注册路径（翻转→回滚+重新投票）：转 C 终局回滚、凹形扩展继续、**待投票确认三项**（转 C 回滚/pnorm-p2 族入 ADR-25 决策 1/族预选规则=物理可行域内拟合集 SSE）。审计通过前 MFD→降级用途仍冻结（通道解禁随投票） |

## 2026-11-18 首轮评审裁决记录（13 项建议）

来源：第三方评审（对 V1.9.3 并入稿 + 五项修订提案 + ADR 文档包）。落实位置见各项"验证条件"列；条文均已同步修订主文档与对应 ADR。

| # | 建议 | 裁决 | 理由 | 验证条件 / 落实位置 |
| --- | --- | --- | --- | --- |
| 1 | 读档等价测试与"丢弃并重新入队"互斥，二选一不得并存 | 采纳（定案"保存并恢复"） | 强读档等价测试（存档读档跑至 M 摘要==直接跑至 M）在丢弃策略下数学上不可能通过 | ADR-NEW-16 决策条文 2；EC-03 / EC-11 同步修订；若改丢弃策略须降级测试为弱形式并另行 ADR 修订 |
| 2 | 过期结果语义缺口（聚合规则、影响区域覆盖动态代价、只保证合法性不保证最优性） | 采纳 | 多事务区间只判断最新一笔会漏相交；2.6 区分拓扑与动态代价，影响区域须全覆盖 | ADR-NEW-17 条文 7/8/9；EC-01 策略块；Spike C 统计验证 |
| 3 | 重算超限前向进展不变量，防止"丢弃-重请求"饥饿 | 采纳 | 持续修路下丢弃+重请求是另一种活锁（永久饥饿） | ADR-NEW-17 条文 10；EC-01 策略块；Spike C 比选兜底候选（局部可行路径/沿用旧路径有效段/标记受阻）后 ADR 定案 |
| 4 | 2.9 浮点禁令边界（位级确定性浮点可经 ADR 批准；Rapier 限定非权威侧或豁免） | 采纳 | SimScalar 选型未定，Spike A 可能需要浮点语义；Rapier 为 f32 基础，直接作权威物理违反条 3 | ADR-NEW-15 条 3；2.9 / 2.1 Rapier 行；豁免须提供位级确定性证据 |
| 5 | LOD 切换确定性（切换条件须为确定性信号） | 采纳 | 中/低精度层改变权威状态，是预算/降级之外的第三个非确定性入口 | ADR-NEW-14 决策第 6 条；2.7 新增条款；切换动作入存档与回放输入 |
| 6 | EC-04 "预算不足或资源压力较高"措辞与确定性触发矛盾 | 采纳 | 资源压力是墙钟类非确定性信号 | EC-04 开头改为"确定性工作量预算超限"；ADR-NEW-14 修订记录 |
| 7 | 流程口径：草案并入（未裁决）+ 可回退 | 采纳 | 提案原文"需经评审后才能并入主文档"与先行并入仅标 Proposed 需口径统一 | 主文档 0.2 / 第 7 章注明草案并入性质；拒绝/延后时按 history/ 提案原件逐条剥离回退；裁决记入本表 |
| 8 | 0.1 映射表补齐实际回填章节 | 采纳 | 映射表漏列 2.3、2.4、2.5、2.6、EC-02、EC-05、EC-06、Stage 0/2A/2B/2C、第 8 章，破坏可追溯性 | 主文档 0.1 映射表逐行补齐 |
| 9 | R2 门禁成本分层（线程数无关性移 R3） | 采纳 | 叠加线程数无关性（3 跑）+注入延迟+读档等价（≥2 跑）= 每 PR 6+ 次全场景，耗时不可接受 | R2=单线程基准+读档等价；R3=线程数无关性（1/2/8 线程+注入延迟）；R2 如需覆盖用缩小场景；实测耗时记录于 docs/benchmarks/ 后复核 |
| 10 | 证据可复现：analysis.py 及输出入仓 | 有条件采纳 → 已落实 | 唯一实验证据不可复现违反证据纪律；当时 analysis.py 未随包提供 | 2026-11-18 已入仓 docs/benchmarks/analysis.py + analysis-output.txt（复核重建版，重跑结论与主文档引用一致） |
| 11 | 2.10 参数表增"最大持续运行 tick 数"行 + 每行定案方法/截止点 | 有条件采纳 | 该行决定中间乘法位宽与漂移口径；tick 频率 10/20/30 Hz 仅为候选，数值仍待项目负责人定案 | ADR-NEW-18 参数基线表（含定案方法/截止点列，Stage 0 前定案）；主文档 2.10 同步；tick 频率漂移决策数据已入仓 docs/benchmarks/tick-rate-drift.py |
| 12 | clippy 禁用按 crate 分层 | 采纳 | 全局禁 HashMap/HashSet 迭代顺序会误伤渲染/UI/遥测 | ADR-NEW-15 验证方法；EC-12；权威 crate 禁、非权威 crate 不禁 |
| 13 | 工作区整理（history/ 归档、主文档转 Markdown、ADR-NEW-01~13 建档排期） | 采纳 | 第 9 章目录结构要求；decision-log 中 01~13 与 14~18 建档不对称 | history/ 归档 V1.9.2 与修订提案原件（回退依据）；docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md 为正式版；ADR-NEW-01~13 独立建档排期见"关联"节 |

注：本表裁决针对评审建议本身。五项修订提案（ADR-NEW-14 至 ADR-NEW-18）的整体裁决已于 2026-11-19 完成（见《2026-11-19 正式裁决记录》）。

## 2026-11-18 二轮修订记录（Spike A 四件套评审 P1~P8 + 设计审计随手修）

来源：Spike A 产物评审（analysis.py / Cargo.toml / lib.rs / REPORT.md）与开工前设计审计的文字级发现。P1~P8 裁决=采纳（文档负责人直接落实，已完成）。

| # | 事项 | 裁决 | 落实位置 |
| --- | --- | --- | --- |
| P1 | lib.rs 须按 cargo 约定放 src/ 布局；Cargo.toml 补 license 字段 | 采纳 | spikes/simscalar-spike/{Cargo.toml, src/lib.rs}（license="MIT OR Apache-2.0"） |
| P2 | 证据链：以"复核重建版"入仓，消除"证据断链" | 采纳 | docs/benchmarks/analysis.py + analysis-output.txt + Spike-A-SimScalar-REPORT.md（含修订记录） |
| P3 | REPORT Q16.48 拒绝理由不准确（±32,768 m 恰容 64 km 图，真正排除 >65.5 km） | 采纳 | REPORT 修订版改为"地图宽度上限 65.5 km（±32,768 m）" |
| P4 | tick 频率口径对齐 2.10 候选 10/20/30 Hz（原脚本 20/60 TPS；60 TPS 下 floor=round 同偏差） | 采纳 | docs/benchmarks/tick-rate-drift.py + tick-rate-drift-output.txt |
| P5 | 漂移线性累积口径挂钩"最大持续运行 tick 数"（+0.7 mm/1e6 → 1e9 ticks ≈ 0.7 m） | 采纳 | REPORT 修订版；2.10 新增行（首轮 #11） |
| P6 | 补 Q24.40 秒累积 0.3 年溢出（原 REPORT 漏列） | 采纳 | REPORT 修订版 |
| P7 | 工具链实测 rustc 1.100.0 vs 文档基线 1.89，待收敛 | 采纳（待决） | 2.1 待复现验证基线；实测记录于 REPORT；基线收敛由负责人定案 |
| P8 | 加 #![forbid(unsafe_code)] | 采纳 | spikes/simscalar-spike/src/lib.rs |
| 二轮随手修 | 2.7 线程分层漏改、EC-03"已列入 R2"、过期策略状态用语（默认候选策略，状态 Proposed）、预算滚动状态注、Stage 1/原始路线图悬空注、Rapier 评估归属排期、主文档新增 6.5 节（ADR-NEW-19/20 索引） | 采纳 | 主文档 V1.9.3（MD + docx 同步，grep 验证 10 处命中） |

## 2026-11-18 外部评审裁决记录（V1.9.3 公开架构评审报告）

来源：用户转发第三方 AI 架构评审报告（2026-11-18）。裁决口径沿用第 7 章：采纳 / 有条件采纳 / 拒绝 / 延后，每项记录理由与验证条件落实位置。

| # | 评审发现 | 裁决 | 理由 | 验证条件 / 落实位置 |
| --- | --- | --- | --- | --- |
| P0-01 | 单 tick 原子 A* 前向进展：预算不足"中止-从头重试"可能永久饥饿 | 采纳 | 我方 25 条审计未登记之新漏洞；前向进展不变量只覆盖路网修改轴 | 债务 #23；ADR-NEW-20 新增"预算让步与前向进展"节（双计数器、A/B/C 比选默认 B、验收 6 条） |
| P0-02 | 预算状态生命周期（累积/公平性/入存档/入摘要/最大等待） | 有条件采纳 | 与债务 #20 同题，扩展为队列权威性与公平性 | ADR-NEW-20 决策补充第 3 条；预算队列入 EC-03 存档 + ADR-NEW-19 摘要纳入范围 |
| P0-03 | 强读档等价未覆盖任务队列/优先级/让步计数 | 有条件采纳 | 强读档等价已实装通过（save-restore-spike N=40→M=100，评审未读到）；增量=队列/优先级/让步状态覆盖 | ADR-NEW-20 验收 4；EC-11 R2 强等价场景扩展 |
| P0-04 | 摘要顶层 Schema 应独立于存档 Schema + domain separation | 采纳 | 我方"共用编码器"口径过宽 | ADR-NEW-19 决策条 2 已收紧（共用编码规则、分顶层 Schema、domain separation） |
| P0-05 | Spike A 证据链（7/7→12/12 记录不一致、1/30 浮点表示、aarch64 缺失） | 有条件采纳 | 7/7→12/12 为分阶段记录非矛盾；1/30 不可精确表示已有结论（dt_raw=143,165,577 黄金值+固定偏差） | 后续报告统一指向同一提交哈希；aarch64 列入 R4 跨平台验证 |
| P0-06 | AABB 宽筛过度重算，建议两阶段 | 采纳 | 即 ADR-NEW-17 备选"逐边校验"的两阶段组合 | ADR-NEW-23 新增"两阶段相交判定"节（窄判按边 ID、指标清单、6 场景矩阵） |
| P1-01 | clippy 禁 HashMap 收窄为"禁依赖迭代顺序结果" | 有条件采纳 | 与我方设计审计 #14 同向 | ADR-NEW-15 条 4 落地口径（#[allow] 豁免+审查；类型级禁用保留第一道护栏） |
| P1-02 | 稳定提交全序键与系统执行顺序分离 | 采纳 | 全序键缺定义为真缺口 | ADR-NEW-15 条 5（候选复合键 (phase_id, system_id, request_id, agent_id, local_sequence)） |
| P1-03 | RNG namespace 注册表与统计质量测试 | 采纳 | 已登记（ADR-NEW-22 遗留项） | 随债务 #15 与 ADR-NEW-22 裁决推进 |
| P1-04 | 代价更新使路径失效与"只保证合法性"语义张力 | 采纳 | 条文 8/9 张力属实 | ADR-NEW-23 两阶段节第 2 条（代价更新默认不触发重算，策略旋钮默认关闭） |
| Q11 | Stage 1 任务清单要素建议 | 采纳 | 即债务 #19 | 要素清单并入重建；Stage 1 开工前完成 |
| Q12 / 6.x | MSRV 三区分语义、依赖安全 | 采纳 | 即债务 #22 / #21 | CI 本轮加 MSRV 1.89 check job；安全项已入债务 #21 |
| 裁决建议整体 | 10 ADR 9 有条件采纳 + NEW-20 暂缓；17/20/23 联动决策组 | 有条件采纳 | 联动组提示采纳；NEW-20 不整体暂缓——方案③保持推荐、附 P0-01 前置验证条件 | ADR-NEW-23 修订记录联动组提示；三项语义一致性验证完成前不单独定案 |

评审总体结论（继续推进、暂不冻结）与我方 Stage 0 Exit Checklist 一致：推进不冻结；ADR 正式裁决前不关闭 Stage 0。

## 2026-11-19 正式裁决记录（五家独立 AI 评审共识）

来源：项目负责人委托五家独立 AI 评审，维护者汇总代录。报告清单：
- A = CitySim-rs-ADR评审裁决-NEW14-23.md（1 采纳 + 9 有条件采纳）
- B = AI-REVIEW-VERDICT-2026-11-19.md（3 采纳 + 7 有条件采纳，附按 issue 拆分的粘贴段）
- C = CitySim-rs-ADR-NEW-14~23-裁决报告.md（2 采纳 + 8 有条件采纳；机制 B 三硬约束与解析上界）
- D = 11.docx（9 有条件采纳 + NEW-20 延后；机制 A/B/C 压测清单）
- E = 重复投递（与前一报告逐字相同），不计独立票；其证据断言不实（虚构 docs/benchmarks/GlobalDigest-test、"ADR-12 边界一致"、benchmark/bench-pathfinder、Pathfinder-degrade.md 等；GlobalDigest 至今为 FNV 占位、无正式测试），证据引用已驳回，裁决方向仅作参考。

- F = 第六份报告（落盘后到达，工程细节最翔实：3 采纳 NEW-15/16/21 + 7 有条件采纳）：有效修正 5 处已并入——①机制 B 不得写为定案（A/B/C 以 G5 实测比选锁定）；②ADR-NEW-17 条文 10 兜底回填 MarkBlocked 定案值；③NEW-19 模块清单含预算队列/让步状态+滚动摘要以实测为准；④NEW-22 统计测试"或书面豁免"+协议版本不符处理随债务 #12；⑤NEW-21 裁决后 2.4 转 Accepted。其过时点：NEW-16 表增补已先行落实、NEW-18"Rust 零实测"与 Step 2 实测不符（仅舍入专项仍缺）。F 另主张严格状态口径（有条件采纳在条件满足前保持 Proposed）——记为替代口径；本记录采用"Accepted（附条件）"（状态词表无中间态，条件=Stage 1/2A 门禁），由项目负责人最终确认。

六家共识：10 项方向全部成立（无一拒绝）；17/20/23 联动组不拆分定案；NEW-21 证据最扎实；NEW-20 机制 B 为默认候选、最终以 G5 压测比选锁定（D"延后"折算为"定案候选、实测验收后实施"，F"不得写为定案"同向）。

| ADR | 正式裁决 | 五家分布（A/B/C/D/E） | 验证条件 / 实施前置 |
| --- | --- | --- | --- |
| NEW-14 | 有条件采纳 → Accepted（附条件） | 有条件/有条件/采纳/有条件/重复 | 声明载体与 L 级映射（债务 #16）；降级触发/恢复一致性实测入 docs/benchmarks/ |
| NEW-15 | 采纳 → Accepted | 有条件/采纳/有条件/有条件/重复 | 条 5 缺省序已补；R4 复验保持待验证标注 |
| NEW-16 | 有条件采纳 → Accepted（附条件） | 有条件/采纳/有条件/有条件/重复 | EC-03 表增补已落实；存档格式随 ADR-NEW-19（债务 #11）；回档验证入 EC-07 |
| NEW-17 | 有条件采纳（联动组）→ Accepted（实施前置） | 联动×5 | 组级验收 G1-G5+两阶段窄判测试覆盖 |
| NEW-18 | 有条件采纳 → Accepted（附条件） | 有条件/采纳/有条件/有条件/重复 | 舍入规则 Rust 专项黄金（截断对比）；CS2 参考值标注"设计参考非官方实测" |
| NEW-19 | 有条件采纳 → Accepted（附条件） | 有条件×4/重复·证据不实 | 黄金测试向量+digest_version bump；实施后以 R2 oracle 验证；**最优先实施** |
| NEW-20 | 有条件采纳（联动组，机制 B 定案）→ Accepted（实施前置） | B+C 组合/B/D 延后/联动 | 三硬约束+ID 分配点条款（债务 #24）；G1-G5+压测清单 |
| NEW-21 | 采纳 → Accepted（无条件） | 采纳/有条件/采纳/有条件/重复 | Q24.40 整数位勘误（结论不变）；转 Verified 外围条件（债务 #22/正式基准/R4 黄金） |
| NEW-22 | 有条件采纳 → Accepted（附条件） | 有条件×4/重复 | namespace 碰撞校验（债务 #25）；统计质量测试 R4 前 |
| NEW-23 | 有条件采纳（联动组）→ Accepted（实施前置） | 联动×5 | 两阶段+代价语义测试（6 场景矩阵）；与 17/20 一并实施 |

联动组（17/20/23）组级验收单（实施前置，挂 Stage 1 寻路开工前）：
- G1 双轴前向进展不变量：路网 churn 轴+预算轴，请求须在固定上限 tick 内完成或进入受控终止；
- G2 机制选择锁定：以 G5 压测数据在 A/B/C 间比选锁定（默认候选 B+三硬约束：放大幅 g 与上限 L≥|V| 常量化+解析上界 k≤⌈log_g(|V|/b)⌉+1、总额不变确定性配额再分配、三计数分离；数据推翻默认时按修订流程切换）；
- G3 让步状态（budget_yield_count/stale_recompute_count/等待 tick/优先级）入 EC-03 存档与 GlobalDigest 纳入范围；
- G4 条文互指对齐静态复核（17/20/23 语义一致性，2026-11-19 勘误后复核）；
- G5 预算轴实测（评审 D 压测清单）：200×200 路网极端长路径下机制 A/B/C 对比——最坏单 tick 耗时、重复展开率、终止率、MarkBlocked 阻塞率、假前向进展场景，数据入 docs/benchmarks/。

Stage 0 判定（按评审 C 折中路径，A/B/C 可关 vs D 不可关）：正式裁决完成后退出条件 6"关键 ADR 已完成评审"满足，**Stage 0 关闭**；遗留债务（#5/#6/#8/#17/#20/#24/#25/#26 等）全部转 Stage 1/2A 开工门禁，不阻塞关闭；联动组验收 G1-G5 挂 Stage 1 寻路开工前。

## Stage 0 工程步骤执行日志

按主文档第 5 章 Stage 0 重点、第 8 章优先级顺序逐步执行；每步一个 git 提交（提交主题见"提交"列）。

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| Step 1 | 补齐 2.10 参数基线（第零优先，负责人定案） | 已完成（2026-11-18） | docs(step1): 2.10 参数基线定案（CS2 参考） |
| Step 2 | Spike A：SimScalar（定案参数重跑 + Rust 范围/漂移验证 + 性能数据 + 数值设计决策 ADR） | 已完成（2026-11-18） | docs(step2): Spike A 收尾——定案核对与 SimScalar 定案建议 |
| Step 3 | Spike B：系统级 RNG（原型 + 确定性测试 + 算法比较 + ADR） | 已完成（2026-11-18） | docs(step3): Spike B——系统级 RNG 原型、9 测试、ADR-NEW-22 |
| Step 4 | Spike C：路网事务原子发布（原型 + 并发测试 + 过期结果统计 + 行走删路比选 + ADR） | 已完成（2026-11-18） | docs(step4): Spike C——路网事务原子发布原型、8 测试、ADR-NEW-23 |
| Step 5 | 存档兼容验证（EC-03 扩充字段 + 读档等价测试） | 已完成（2026-11-18） | docs(step5): 存档兼容验证——EC-03 最小集原型、9 测试、读档等价模板 |
| Step 6 | 测试基础设施（EC-07 sim-test-support + 确定性注入） | 已完成（2026-11-18） | docs(step6): 测试基础设施——sim-test-support 六模块、11 测试、读档等价模板 |
| Step 7 | 基础 CI（R0/R1 门禁骨架 + clippy 分层实测） | 已完成（2026-11-18） | docs(step7): 基础 CI 骨架 + clippy 分层实测通过 |
| Step 8 | 依赖与工具链验证（rust-toolchain 锁定 + 五件套编译验证） | 已完成（2026-11-18） | docs(step8): 依赖与工具链验证——五件套、MSRV 发现、安全审计 |
| Step 9 | 核心工程 workspace 稳定编译 + Stage 0 退出条件核对 | 已完成（2026-11-18） | docs(step9): workspace 稳定编译 + Stage 0 退出条件核对（5/6，ADR 评审待决） |

## 2026-11-19 决策投票记录（状态口径 / Rust 基线）

- 票面归档：docs/architecture/review/archive/决策-投票-2026-11-19.docx（7 家 AI 投票）。
- **决策 1（ADR 状态口径）：A 全票 7:0** —— "Accepted（附条件）"为 Accepted 的正式注记写法（0.2 注记规范已写入主文档）。配套纪律：①注记必须引用条件核销清单条目或债务编号；②条件绑定债务编号+截止点；③条件未清不得开工对应模块；④转 Verified 必须附实测证据；⑤条件核销以本节清单 checkbox 逐条核销（维护者执行，核销附证据路径）。
- **决策 2（Rust 工具链基线）：B 6:1** —— 文档基线 = 开发工具链 = CI 主工具链 = **Rust 1.100.0**；Rust 1.89 为 Bevy 0.18.1 MSRV 兼容性参考；msrv-check job（1.89 check）保留为门禁；ordered-float 5.4.0 钉版维持至 MSRV 调整时复核（少数派与部分多数派主张解除钉子，随 MSRV 调整一并复核）；MSRV 保证目前为 check 级，test 级验证随首个发布版本补。**债务 #22 以本决策关闭**（2.1 基线句已改写）。
- **条件核销清单**（转 Verified 前逐条核销；核销须附证据路径）：

- [x] ADR-NEW-16：EC-03 表增补（三类 ID 高水位+让步状态）——已落实（2026-11-19）
- [x] ADR-NEW-17：条文 10 兜底回填 MarkBlocked——已落实（2026-11-19）
- [x] ADR-NEW-21：工具链收敛记录——已关闭（见决策 2，2026-11-19）
- [ ] ADR-NEW-14：各系统计量单位/每 tick 上限阈值实测回填
- [ ] ADR-NEW-14：R2/R3 分层后门禁实测耗时记录入 docs/benchmarks/
- [ ] ADR-NEW-14：预算滚动状态生命周期回填（债务 #20）
- [x] ADR-NEW-15：crate 分层 disallowed-types 落地验证不误伤非权威 crate——已核销（2026-11-19，T1：crate-authority-list.md + 6 权威 crate clippy.toml + dep_graph_check CI 门禁；sim-view/sim-test-support 不受限，7 crate clippy CLEAN）
- [ ] ADR-NEW-15：全序键字段按实际冲突模型定稿回填条文 5
- [ ] ADR-NEW-16：永久兼容边界定义
- [x] ADR-NEW-16：存档序列化格式定案（债务 #11，随 ADR-NEW-19）——已核销（2026-11-19，T6/T7：sim-codec canonical 信封（CSIMCOD1|sim-save|Save）+ Schema v1（sim-save SCHEMA_VERSION=1）+ BLAKE3 尾摘要实施）
- [ ] ADR-NEW-17：细则取值随 NEW-23 回填（兜底已回填；相交粒度/重算上限待 G5 数据复核）
- [ ] ADR-NEW-17/20/23：联动组一致性验证 G1-G5 + 压测清单（200×200 极端长路径）（G3"让步状态入 EC-03/GlobalDigest"已落实：sim-save budget_yield_count/stale_recompute_count 入档入摘要（2026-11-19 T7）；余 G1/G2/G4/G5 待 Stage 2A）
- [x] ADR-NEW-18：舍入规则 Rust 专项验证（误差不单向累积）——已核销（2026-11-19，T2：simscalar 12 测试含 ties-away 舍入与 drift_golden_30tps（恒定 +7 raw/tick 固定偏差非加速累积）+ docs/benchmarks/tick-rate-drift 数据）
- [ ] ADR-NEW-19：BLAKE3 跨平台位级一致性 Spike
- [x] ADR-NEW-19：编码器选型定案（含黄金测试向量、digest_version bump 规则）——已核销（2026-11-19，T6：自定义 canonical 编码器实施（sim-codec，否决 bincode）、黄金向量 3 组入仓 docs/benchmarks/globaldigest-golden-vectors.txt、DIGEST_VERSION=1 及 bump 规则入 ADR-NEW-19 条文）
- [ ] ADR-NEW-19：纳入模块清单定稿（含预算队列/让步状态）（Stage 1 进展：sim-save 摘要已含预算队列/让步状态（G3）；全模块清单待模拟核心各模块落地定稿）
- [ ] ADR-NEW-19：滚动摘要去留实测定案（性能 1~5 ms/tick 为估计值）
- [ ] ADR-NEW-20：验收 6 项实测（含让步路径强读档等价）
- [x] ADR-NEW-20：G5 压测后 A/B/C 机制锁定（默认候选 B+三硬约束）——已核销（2026-11-19，Phase 3：docs/benchmarks/G5-Verdict-REPORT.md，机制 B 默认候选确认、A 为低浪费切换、C 单独不可用；切换阈值数值另立债务 #27）
- [ ] ADR-NEW-20：每 tick 节点上限、最大等待 tick 上限与受阻衔接定值（随 NEW-14 阈值）
- [ ] ADR-NEW-20：请求/事务 ID 单一分配点定案（债务 #24）（Stage 1 进展：T3/T7 已实装集中单一分配点（CommandIdAllocator / Allocators）；tick 内固定相位取号规则随调度器（Stage 2B）钉死）
- [ ] ADR-NEW-21：R4 跨平台黄金值复验（dt_raw、drift_golden）（跨平台待 R4）
- [ ] ADR-NEW-21：正式基准替代冒烟值（2.5 ns/op 不得作门禁）
- [x] ADR-NEW-22：namespace 注册表+注册期碰撞校验（债务 #25）——已核销（2026-11-19，T5：sim-rng NamespaceRegistry 注册期 64 位碰撞拒绝 + 构造碰撞测试）
- [ ] ADR-NEW-22：TestU01/PractRand 统计测试或书面豁免（SplitMix64 统计充分性）
- [ ] ADR-NEW-22：协议版本不符处理随 EC-03 兼容矩阵（债务 #12）回填
- [ ] ADR-NEW-23：6 场景矩阵验收指标实测入 docs/benchmarks/（误判率/重算率/阻塞率/等待 tick）
- [ ] ADR-NEW-23：PartialPath/UseValidPrefix 启用前清债务 #18
- [ ] ADR-NEW-24：Edge 有向语义入数据字典（方向性/平行边策略/代价下限不变量）（债务 #8 剩余，Stage 2B 前）
- [ ] ADR-NEW-24：机制 B→A 切换阈值定案（债务 #27，Stage 2B 并发压测后）——**进展（2026-11-19 V4）**：多场景基线已实测（dup_rate 证伪为判据、候选判据=单次尝试展开>4096 或让步≥8 切机制 A），**数值定案随 G2 锁定**后核销本行
- [x] ADR-NEW-24：事务重试 tx_id 语义（债务 #28）+全局每 tick 寻路配额（债务 #29）——**2026-11-19 Stage 2B 前置定案**（#28=沿用原 tx_id 全序重排+重试上限 3 转受阻；#29=TickBudget{remaining=Q}，B_k≤min(b·2^k,L,remaining)，Q=65536 **已定案**（2026-11-19 决策3 投票 1a，9:1；单一来源 `sim-pathfinding::TICK_QUOTA`，2026-11-20 X5 钉死））；#28 实装仍待（2026-11-20 审计更正见债务行）
- [x] ADR-NEW-24：A* 细则补遗（h 一致性不变量/L=|V|×reopen 上界/前驱裁决明文/CSR u32 边界）（债务 #30）——**已核销（2026-11-19 V3）**：astar_edge_cases 5 测试（reopen 差分 13 vs 1010、等代价前驱=先发现保留、乱序注入稳定、CSR u32=179,400 边、h 一致性边界书面化），docs/benchmarks 证据=crates/sim-pathfinding/tests/astar_edge_cases.rs
- [x] ADR-NEW-24：让步路径强读档等价专项 + 并发队列压测 + R3 线程无关（1/2/8 线程+混沌注入）——**2026-11-19 Stage 2B 完成**（证据：docs/benchmarks/Stage-2B-Exit-CHECKLIST.md、Stage2B-concurrent-stress.txt、sim-save `yield_state_strong_readback`）
- [ ] ADR-NEW-24：R4 跨平台摘要一致性（移交 R4 发布前门禁，不属 Stage 2B 范围）
- [ ] ADR-NEW-25：债务 #31 复判收口——Set D 复测（X1-X4 整改后）**判定翻转：转 C 不成立**（pnorm-p2 物理可行两拓扑三区全 PASS；证据 commit ce4a705，docs/benchmarks/mfd-3a-setd.txt）；待投票确认三项（族选型/选族规则/回滚确认）后核销本行；此前 MFD 不作降级用途

## Stage 1 执行日志

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| S1-1 | Stage 1 任务清单重建（docs/architecture/Stage-1-Task-List.md，T1~T9、退出条件、执行顺序） | 已完成（2026-11-19） | 本次提交 |
| S1-2 | T1 workspace/crate 划分骨架（2.2 分层、权威/非权威清单、clippy 分层） | 已完成（2026-11-19） | docs(stage1): T1——7 分层 crate 骨架、权威清单、依赖方向门禁 |
| S1-3 | T2 SimScalar 正式 crate（ADR-NEW-21） | 已完成（2026-11-19） | docs(stage1): T2——SimScalar 正式 crate（12 测试，Spike 黄金一致） |
| S1-4 | T3 时间与命令日志（2.9 条 7/8） | 已完成（2026-11-19） | feat(stage1): T3——sim-core 权威时间与命令日志（7 测试） |
| S1-5 | T4 Agent 身份（2.3 高水位） | 已完成（2026-11-19） | feat(stage1): T4——sim-agent AgentId 高水位（5 测试） |
| S1-6 | T5 RNG 协议实装（ADR-NEW-22 + namespace 注册表） | 已完成（2026-11-19） | docs(stage1): T5——RNG 协议 crate（12 测试含碰撞校验，债务 #25 落实） |
| S1-7 | T6 canonical 编码器 + GlobalDigest（ADR-NEW-19，最优先上游） | 已完成（2026-11-19） | docs(stage1): T6——sim-codec/sim-digest（13 测试，BLAKE3 黄金向量入仓） |
| S1-8 | T7 存档/读档 EC-03 实装（保存并恢复 + 强读档等价入 R2） | 已完成（2026-11-19） | feat(stage1): T7——sim-save 存档实装（10 测试，BLAKE3+canonical） |
| S1-9 | T8 错误分层骨架（EC-08） | 已完成（2026-11-19） | docs(stage1): T8——sim-error 错误分层骨架（5 测试 + 错误码规范） |
| S1-10 | T9 确定性契约测试接入（2.9 八条证据表 + R2 场景骨架） | 已完成（2026-11-19） | feat(stage1): T9——2.9 八条证据表 + R2 场景骨架（4 测试） |

## Stage 2A 执行日志

| 阶段 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| Phase 1 | 定案前置：ADR-NEW-24（CSR 定案、A* 确定性细则、机制 B 参数、Stage 2A 退出判据、取消/冲突语义） | 已完成（2026-11-19） | docs(stage2a): ADR-NEW-24 草案（f205722） |
| Phase 2 | 实施：sim-network（CSR+原子事务+ADR-NEW-23 两阶段判定，9 测试）+ sim-pathfinding（确定性 A*+机制 B 预算让步+双计数器，7 测试） | 已完成（2026-11-19） | feat(stage2a): sim-network + sim-pathfinding |
| Phase 3 | G1-G5 联动组 200×200 压测（机制 A/B/C 对比） | 已完成（2026-11-19） | docs(benchmarks): G5-stress-200x200.txt + G5-Verdict-REPORT.md（A=7 ticks 零浪费/B=4 ticks 浪费 0.53/C=100% 阻塞；B 上界 11 实测 4 ✅） |
| Phase 4 | Stage 2A 退出核对 + ADR-NEW-24 走第 7 章裁决 | 已完成（2026-11-19：退出核对 5/5；第二轮外部评审 8:0 有条件采纳→Accepted（附条件）） | docs(benchmarks): Stage-2A-Exit-CHECKLIST.md + G5-churn-200x200.txt（重算率 0.0027 PASS）+ 决策2.docx 八家票面 |

## 2026-11-19 第二轮评审裁决记录（ADR-NEW-24，八家独立 AI 评审）

票面：review/archive/决策2-ADR24-投票-2026-11-19.docx（8 家全部署名：MIMO V2-PRO、DEEPSEEK V4.1 FLASH、豆包专家、QWEN4-MAX、CHATGPT、GEMINI、GROK-快速、Claude）。评审材料包=CitySim-rs-ADR24-REVIEW.zip（含 MANIFEST 数据口径说明）；任务书=review/AI-REVIEW-BRIEF-ADR24.md；回传渠道=issue #24。

| # | 评审 | 裁决 | 关键要求（摘要） | 核对/落实 |
| --- | --- | --- | --- | --- |
| 1 | MIMO V2-PRO | 有条件采纳 | 重算率分母应为请求数（3/200=0.015）；0.53 是 g=2 结构性水平（单请求重复率上界 (2^k−1)/(3·2^{k−1})→2/3），切换触发候选=最坏单 tick 耗时 SLO+dup_rate>0.7；"2|V| 实为总展开上界，浪费<|V|"措辞修 | 采纳：判据口径/阈值过渡口径/措辞回填 ADR-24 |
| 2 | DEEPSEEK V4.1 FLASH | 有条件采纳 | 阈值写入 ADR、Edge 入典、R3/R4+并发队列、让步强读档专项；decision-log 链接访问失败 | 采纳；decision-log 与外部材料一致性由维护者核对无冲突 |
| 3 | 豆包专家 | 有条件采纳 | Edge 入典关债务 #8；让步计数是状态非配置；BinaryHeap 替代名录（有序向量/稳定二叉堆）；1M 节点整表重建尖峰 | 采纳：让步计数语义澄清+替代名录入 ADR-24 修订 |
| 4 | QWEN4-MAX | 有条件采纳 | **两条代码缺陷坐实**：(a) wrapping_add 违反 ADR-NEW-21 溢出契约（h/g/f+参考 Dijkstra 对拍盲视）；(b) 回溯父边取 min edge id 而非松弛实际边，平行边下代价与边集不符、污染 stale 窄判 | 全部采纳并已修复（2026-11-19）+2 回归测试（parallel_edge_reconstruction_is_exact、overflow_saturates_deterministically） |
| 5 | CHATGPT | 有条件采纳 | h 一致性不变量；L=|V|×reopen 二选一+reopen 测试；open 集前驱裁决规则；CSR u32 容量边界；交错场景强等价 | 采纳：登记债务 #30 |
| 6 | GEMINI | 有条件采纳 | 并发队列+复杂路网压测（障碍/非均匀权）；阈值 P95>0.65 或耗时尖峰；Edge costs 按有向边索引 | 采纳：压测入 Stage 2B 门禁 |
| 7 | GROK-快速 | 有条件采纳 | Edge 入典；阈值 ≥1.0；判据 Proposed→Accepted 保留 2B 限定 | 采纳 |
| 8 | Claude | 有条件采纳 | 全局每 tick 寻路配额 B_k≤min(b·2^k,L,剩余配额)（L=|V| 与 ADR-14 条 3 每 tick 上限冲突）；不可达预检；"g 相等保留先发现前驱"明文；事务重试上限；补测清单 | 采纳：配额登记债务 #29、细则补遗 #30，其余回填 ADR-24 修订 |

**共识（8/8 有条件采纳，无拒绝/延后）**：Edge 有向语义入典、阈值定案或书面过渡口径、让步强读档专项→Stage 2B、R3/R4+并发队列→Stage 2B；高频=重算率分母统一（4）、事务重试 tx_id（4）、L=|V|×reopen 上界（4）。

**裁决：有条件采纳 → Accepted（2026-11-19，附条件）**（决策 1 全票口径注记；条件绑定债务 #8/#27/#28/#29/#30 与 Stage 2B 硬门禁（让步强读档专项、并发队列压测、R3/R4））。

工程修复（2026-11-19）：
1. **溢出契约**（QWEN-1）：h_manhattan 改 i128+饱和收敛、g/f 累加改 saturating_add、参考 Dijkstra 同步；回归 overflow_saturates_deterministically。
2. **父边精确重建**（QWEN-2）：parent 记录 (父节点, 松弛实际边 ID)、回溯不再取 min edge id；回归 parallel_edge_reconstruction_is_exact（平行边下返回边集代价和==报告 cost）。
3. **判据口径**：重算率定义=过期重算请求占比（分母=请求总数），实测 3/200=0.015；0.0027（3/1100）保留标注为 stale 检查口径。
4. **阈值过渡口径**：0.53 记为 g=2 结构性观测基线（不触发/误触发风险），切换触发候选=最坏单 tick 耗时 SLO 或 dup_rate 超阈；数值待 Stage 2B 并发压测定案（债务 #27）。**2026-11-19 V4 实测更新**：dup_rate 被证伪为判据（三场景 max 0.664≈解析上界 2/3）；候选判据改确定性工作量单位=单次尝试展开 >4096 或让步 ≥8 次切机制 A（open 353/3 不触发，obstacle 10228/8、maze 17047/9 触发），数据 docs/benchmarks/G5-threshold-baseline.txt。

过时/澄清（不采）：Claude 引 ADR-23"重做展开消耗预算问题不存在"句已于 2026-11-18 勘误修复（属过时信息）；Claude"不可达走满档总展开≈105k"推算前提不成立（open 耗尽即 Unreachable 提前终止，不走满让步档）；budget_yield_count=单请求计数器终值（完成清零语义），累计让步≈405=605−200（MANIFEST 口径）；G5 churn 删边非成对（MANIFEST 口径，成对删边列入 Claude 补测清单）。

## Stage 2B 执行日志

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| B1 | 前置定案（债务 #28 事务重试 tx_id、#29 全局每 tick 寻路配额）+ TickBudget 实装 + 让步强读档专项 | 已完成（2026-11-19） | ffd4516 + B1c 提交 |
| B2 | 确定性并行框架（2.9 条 6：快照读取+稳定键分片+串行提交）+ 线程无关 harness | 已完成（2026-11-19） | feat(stage2b): B2——sim-parallel 确定性并行框架（47ab0e2，含 CI 矩阵补 Stage 1 六 crate） |
| B3 | 线程无关 1/2/8 线程+注入延迟测试 + 并发队列压测 | 已完成（2026-11-19） | feat(stage2b): B3——线程无关场景测试 + 并发队列压测（af9427c，1/2/8 线程摘要一致 PASS） |
| B4 | 退出核对（主文档 Stage 2B 四条退出条件）+ 文档同步 | 已完成（2026-11-19） | docs(stage2b): B4——退出核对 4/4、Stage-2B-Exit-CHECKLIST |

## Stage 2C 执行日志

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| C1 | MFD 验证方案起草（ADR-NEW-25：模型形式候选、三项独立验证、T-Bench-1 可复现场景、确定性判据） | 已完成（2026-11-19） | docs(stage2c): C1——ADR-NEW-25 草案 |
| C2 | 背压/预算契约（ADR-NEW-14 阈值候选提案 + L1/L2/L3 降级映射，债务 #16） | 已完成（2026-11-19） | docs(stage2c): C2——预算契约细化（ADR-NEW-14） |
| C3 | 实施（sim-traffic 最小实现 + 寻路背压 + 确定性降级验证 + T-Bench-1） | 已完成（2026-11-19） | feat(stage2c): C3a——BudgetRegistry（sim-core 12 测试）；C3b——sim-traffic MFD+T-Bench-1（4 测试+基准数据） |
| C4 | 退出核对（主文档 Stage 2C 六条退出条件）+ 文档同步 | 已完成（2026-11-19） | docs(stage2c): C4——退出核对 6/6、Stage-2C-Exit-CHECKLIST |

## 决策3 落盘执行日志（Stage 2C 投票修复，2026-11-19）

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| S1 | 代码修复批 A（fit_triangle 最大割线斜率+下降支辨识标志、fold_digest 补全权威状态字段、T-Bench-1 窗口均值+稳态判据+双向扫描+完整输出） | 已完成（2026-11-19） | fix(stage2c): S1——决策3 代码修复批 A |
| S2 | 代码修复批 B（注入顺序无关性测试、budget 聚合口径逐系统取最严+连续 N tick 触发、拓扑/代价事务分开上限） | 已完成（2026-11-19） | fix(stage2c): S2——决策3 代码修复批 B |
| S3 | 阈值回填 ADR-NEW-14 转定案（1a/1c 通过值 + 1b/1d 修改后值 + "100× 余量"措辞更正） | 已完成（2026-11-19） | docs(stage2c): S3——阈值投票定案回填 ADR-NEW-14 |
| S4 | ADR-NEW-25 修订（3a 独立参照/3b 对齐/3c 实装/协议与实现一致化/权威性边界拆分）转 Accepted（附条件） | 已完成（2026-11-19） | docs(stage2c): S4——ADR-NEW-25 修订与正式裁决 |
| S5 | decision-log 裁决记录（10 票表 + 2 份 ADR-NEW-24 补充裁决 + 债务更新） | 已完成（2026-11-19） | docs(stage2c): S4+S5+S6——决策3 落盘收口 |
| S6 | issue #25 回贴收口 + 终验 | 已完成（2026-11-19） | docs(stage2c): S4+S5+S6——决策3 落盘收口 |

## 2026-11-19 决策3 投票裁决记录（Stage 2C 两项决策，10 家署名票）

- 票面来源：C:\Users\wenyu\Desktop\决策3.DOC（用户组织投票，按评审常驻规矩署名制）。
  10 家署名：DeepSeek-V4.1flash、MiMo（Xiaomi）、豆包专家（材料读取失败自标保留票）、
  Kimi Chat（Moonshot）、千问Go!（qwen4-max）、MiniMax-M3.1-Flash（minimax Code/Mavis，
  代码级最深：材料包与仓库逐字节 diff+行号级）、Claude（Anthropic）、Gemini、Grok 4.5
  （xAI）、GPT-6。另有 2 份顺带 ADR-NEW-24 补充裁决（有条件采纳；一份附 GPT-6 后，
  一份未署名按规矩仅记录不计票）。
- 决策 1（阈值，10 票）：1a Q=65536 **通过**（9:1；"100× 余量"措辞按 3 家要求更正为
  "对实测 worst 398 的保险丝，真实上界≈2|V| 由机制 B 兜底"）；1b 70/90/100 **修改后定案**
  （聚合口径逐系统取最严 + 连续 2 tick 触发/恢复门，4 家要求）；1c 恢复 50% **通过**
  （闭区间触发 >=、开区间恢复 <，附边界测试已实装）；1d 事务上限 **修改后定案**
  （拓扑/代价分开各 1024 + 单笔 edge_ops 8192，Claude/GPT-6 要求）。全部数值于
  ADR-NEW-14 C2 节回填定案（S3）。
- 决策 2（ADR-NEW-25，10/10 有条件采纳，零拒绝零延后）：共识硬伤 11 项，其中代码面
  5 项已修复回归（S1/S2：拟合重写 31× 失真消除、fold_digest 补 goal/next_id、窗口
  均值+稳态判据、双向扫描滞回、注入顺序无关性）、协议面 6 项已修订入 ADR（S4：
  模型形态辨识二选一、权威性三层拆分、3a 独立参照防同义反复、3c 生产代码路径条件、
  协议与实装一致化、3a/3b/3c 独立指标文件）。ADR-NEW-25 → Accepted（附条件）。
- 质量记录：MiniMax-M3.1-Flash（逐字节材料校验+行号级，发现阈值形同虚设/3c 无代码
  路径）、Kimi Chat（三点估计≠最小二乘首发）、千问Go!（fold_digest 漏字段假通过风险）。
  豆包专家票自标"材料读取失败、不构成正式评审结论"，按保留票记录。
- ADR-NEW-24 补充裁决 2 份均为有条件采纳（Q2 阈值建议 ≥1.0/1.5、Edge 入典、reopen/
  异常补测），与第二轮 8:0 结论一致，条件并入债务 #27/#30 执行口径。

## ADR-NEW-25 条件与债务执行日志（V1–V5，2026-11-19）

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| V1 | 3a 独立参照验证（拟合 A/试点 B/确认 C 三集+多 seed 平均、三区判据复核回填、mfd-3a-reference.txt） | 已完成（2026-11-19） | docs(stage2c): V1——3a 独立参照验证 || V2 | 3c 生产代码路径（MFD→降级信号接入 BudgetRegistry）+专用耦合线程无关测试 | 已完成（2026-11-19） | docs(stage2c): V2——需求压力降级通道（ADR-NEW-26 草案）+耦合线程无关测试 |
| V3 | 债务 #30 A* 细则补遗测试（reopen 差分/等代价多路径/乱序注入/CSR u32 边界） | 已完成（2026-11-19） | docs(stage2c): V3——A* 细则四组边界测试（astar_edge_cases 5 测试） |
| V4 | 债务 #27 机制 B→A 切换阈值基线（障碍/迷宫场景 dup_rate 多场景实测→候选提案） | 已完成（2026-11-19） | docs(stage2c): V4——G5 阈值基线三场景实测（dup_rate 证伪、候选判据=单次展开 4096/让步 8） |
| V5 | 条件核销清单更新+状态迁移+终验 | 已完成（2026-11-19） | docs(stage2c): V5——条件核销（#30）、#27 基线进展、ADR-25 条件进展表、goal 收官 |

V1 实测结论：确认集 C 饱和区 102‰ PASS（判据 250‰）、自由流区 190‰ FAIL（判据 100‰）、
转折区 521‰ 记录；根因=模型形式凹形缺陷（债务 #31）；3a 状态=部分通过，转 Verified
条件仍未清（如实记录）。判据三区分立定案回填 ADR-NEW-25 3a 节。

## 2026-11-19 决策4 投票裁决记录（债务 #31，10 家署名票）

票面归档：docs/architecture/review/archive/（决策4.MD 原件随本轮提交入库）；回传渠道 issue #26。
裁决分布：**A（凹形扩展）7 : B（两段式）2 : C（放弃）1**，无 D。定案=A 系凹形扩展 + 吸收 B 家
k_turn 显式衔接参数（DeepSeek/Kimi/Grok 意见）。

| # | 评审人（署名） | 裁决 | 关键要求 / 立场 |
| --- | --- | --- | --- |
| 1 | DeepSeek-V4.1 | A>B>D>C | 凹形族写死入决策 1；k_turn 衔接保留；demand_pressure 分母复核 |
| 2 | MiniMax-M3.1-Flash | C>D>B>A（增设 E） | 根因=微观基准无交通物理语义；15 条要求（判据去自指/门禁脱钩/文码不一致/留出集耗损等）；异议如实保留 |
| 3 | MiMo（Xiaomi） | A>D>B>C | 定点友好凹形（双曲/指数饱和）；判据-模型解耦预注册；k_c 操作性定义 |
| 4 | 豆包（Doubao） | A>B>C>D | 5 族实测 33–111‰；函数族约束写死；取消"把约束当估计"；转折区硬判据；门禁对齐 |
| 5 | 千问Go!（qwen4-max） | A>B（C 预注册回退） | 候选族受数据约束+预注册；新确认集（C 耗损）；k_ref 物理化版本化；MANIFEST 口径勘误 |
| 6 | Kimi Chat | B>A>C>D | 有理饱和复算 C 各点 ≤15‰；B 须三段非两段硬接；若平滑选 A 只许有理饱和式 |
| 7 | Claude Sonnet 5.5 | A>C>B>D | 双曲线复算 C 最大 45‰；定点 +−×÷ 红线；Set D+第二拓扑；终止条件=FAIL→转 C |
| 8 | Gemini | A>B>C>D | 定点凹形+可选下降支；ADR-26 分母改特征密度；CTM/LWR 二阶补齐 |
| 9 | GPT-6 | A（有条件）>D>B>C | 模型扩展与权威性解禁分开裁决；C 禁参与选择+逐 seed 披露；let _=count 缺陷 |
| 10 | Grok 4.5 | B>A>C>D | 两段式+显式 k_turn；下降支出现再升 A |

共识硬伤处置（10 项）：①三区边界自指→判据去自指（k_turn 预注册 fixture，ADR-25 3a (a)）；
②转折区免检 521‰→硬判据 ≤250‰（(b)）；③demand_pressure_bp(k_c)→k_ref 特征密度
（q≥90%·q_max 密度，版本化入档；fail-safe 改向 k_ref==0→0）；④q_model 双实现→公开
q_model_permille、删 mfd3a 副本；⑤门禁脱钩→判据常量单一来源（FREE_FLOW_CRIT_PERMILLE=100
等 4 常量），simulation 测试改判据 100 并 #[ignore]（债务 #31 Set D 复测后恢复）；
⑥mfd_coupling `let _ = count`→assert_eq!(count, active_count) 参与被测信号；⑦留出集耗损
→Set D 新种子不相交+第二拓扑（终止条件 FAIL→转 C）；⑧函数族约束=q(0)=0/零密度斜率自由
估计越界判失败/单调凹/幂律排除；⑨MiniMax 根因异议与 E（修微观基准）如实保留=少数派记录；
⑩Claude 基准漂移疑云→已核对 main mfd-tbench1.txt v_free=1000‰，疑云不成立（31125 为修订前
历史值）。另：简报口径勘误（偏差 134‰/519‰ 绝对口径、凹形起始 k≈34–54‰）。

代码整改（本轮落盘）：sim-traffic 判据常量 4 个公开、q_model_permille 公开、
demand_pressure_bp(k_ref) fail-safe 0、simulation 测试 ignore 门禁、mfd3a 判据常量化+删副本、
mfd_coupling 断言聚合一致；workspace 全绿（0 失败，1 ignore=有意门禁）。

## 债务 #31 执行面（W1–W3，2026-11-20）

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| W1 | 凹形有理饱和拟合器（RationalSaturationFit/q_rational_permille/fit_rational_saturation_detailed=自由估计+越界披露+物理可行解回退/evaluate_rational_fit；5 测试含合成恢复 (200,250)） | 已完成（2026-11-20） | feat(stage2c): W1——凹形有理饱和拟合器（5d4ae15） |
| W2 | Set D 复测（预注册 fixture：种子 0x00C0_FFEE_2222_0000+i 与 V1 不相交、拟合 2→256/留出 3→384 交错 8 档、8 种子平均、40×40+64×64、260 tick 末 60 窗口、稳态仅诊断）——两拓扑自由流 148/142‰ FAIL、转折 70/137‰、饱和 28/44‰ PASS；自由估计斜率 1242/1271‰ 超物理界（约束绑定） | 已完成（2026-11-20） | feat(stage2c): W2——Set D 复测（本次） |
| W3 | 按预注册终止条件转 C 落盘：ADR-NEW-25 终局注、ADR-NEW-26 通道冻结注、债务 #31 核销、mfd-3a-setd.txt 入仓 | 已完成（2026-11-20） | 同上 |

**终局裁决（2026-11-20）**：Set D 复测 FAIL → **转 C**——MFD 永久仅作非权威统计展示，
不作降级/背压触发依据；ADR-NEW-26 需求压力通道冻结（不得接入权威降级路径）。
如实记录两点：① 稳态诊断显示持续注入下高密度档不稳态（40×40 仅 1/8、64×64 0/8 稳态），
② 自由估计初始斜率超 1 车/边/tick 的物理界——即决策4 MiniMax E 议题（微观基准无交通
物理语义）的实证；E 项修基立项可重开本判定（须重新投票）。
**复判追记（2026-11-20，债务闭合审计整改后）**：本段转 C 的证据基础被 7 家审计确认
存在四项 P0——q>k 测量口径误差地板（上述"实证"系口径产物，非 E 议题证据）、判定
边界自指、"越界判失败"口径相反、证据与源码不同源；X1–X4 整改后复测**判定翻转
（转 C 不成立，见 X4 行与债务 #31 行）**。本段终局裁决自追记起失效，待投票确认。

## 债务闭合审计整改执行日志（X1–X6，2026-11-20）

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| X1 | P0 口径/判定修复（sim-traffic）：k/q 同群体采样（last_step_population）+q≤k fail-fast、ZoneBands 预注册绝对带去自指、越界判失败口径（binding 仅诊断）、目标判据同构（相对误差平方和）、数值安全（u128/除零）、证据 commit 标注（SETD_COMMIT）、稳态敏感性（back-half） | 已完成（2026-11-20）| fix(stage2c): X1——审计 P0 修复 |
| X2 | 预算修复（sim-pathfinding）：Unreachable 携带 expanded 扣预算、让步计数分离（budget_yield_count/ quota_yield_count）、FindOutcome::expanded()、回归测试 2 个 | 已完成（2026-11-20）| fix(stage2c): X2——Unreachable 预算入账+让步计数分离 |
| X3 | 广义凹形族并行辨识（p-norm p=2：q=q_max·k/sqrt(k²+k0²) 定点 isqrt、拟合、评估、4 测试）+评估泛化核 evaluate_q_fn 消除双实现+Set D 报告双族并行（防 gaming：族选择只看拟合集 SSE 锁定于留出评估前） | 已完成（2026-11-20）| fix(stage2c): X3——p-norm p=2 族并行辨识 |
| X4 | Set D 复测（修后重跑，证据 commit ce4a705 带入文件头）：**判定翻转**——q/k 口径修复后两拓扑三区全 PASS（40×40：83/118/56‰；64×64：80/112/89‰；原自由流 148/142‰ FAIL 系测量 bug 误差地板，豆包复算证实）；rational 族自由估计斜率 1095/1092‰ 越界=族不适配（条件⑧淘汰）；**pnorm-p2 族物理可行（932/944‰）且两拓扑三区全 PASS（83/189/12、48/120/20‰）→"凹形不可救→转 C"不成立**。按豆包 A2 预注册路径：翻转→回滚转 C 终局+凹形扩展继续+**重新投票**（族选型 pnorm-p2/物理可行域选族规则/转 C 回滚确认） | 已完成（2026-11-20）| fix(stage2c): X4——Set D 复测翻转 |
| X5 | 状态/文档卫生：#27 保持候选、#28 改待实装、Q=65536 钉死、README/主文档/核销清单/ADR-26 公式 | 已完成（2026-11-20） | fix(stage2c): X5——状态修与文档卫生（Q 单一来源、#27 保持候选、#28 待实装） |
| X6 | 审计裁决记录（7 家）+issue #27 回贴+终验 | 已完成（2026-11-20） | fix(stage2c): X6——审计裁决入账+issue #27 回贴 |

## 2026-11-20 债务闭合审计裁决记录（7 家署名，审计1.MD）

| # | 报告者 | 总体 | 分项要点 |
| --- | --- | --- | --- |
| 1 | DeepSeek-V4.1 | 有条件入账 | A 有条件（预注册无独立时间戳）/B 有条件（h 缺动态代价）/C 需重做/D 有条件/E 有条件/F 不通过（ADR 状态不同步） |
| 2 | MiniMax-M3.1-Flash | 有条件入账（4 前提） | A 有条件（预注册条款未执行+越界口径相反+"E 实证"须撤回）/B 通过/**C 不通过**/D 有条件（#28 未实装）/E 有条件 14 条/F 有条件 |
| 3 | MiMo | 有条件入账（5 前提） | A 有条件（暂定入账，修口径复测后处置）/B/C/D 有条件（发现 Unreachable 漏记）/E/F 有条件 |
| 4 | Gemini | 有条件入账（4 项） | A 有条件（采样时序+稳态）/B 有条件（哑坐标）/C 有条件/D 有条件（让步计数污染）/E/F 有条件 |
| 5 | GPT-6 | **需重做** | A 有条件（"执行终止条件≠证据质量验证"）/B/C 有条件/**D 不通过（Unreachable 漏记）**/**E 不通过（11 条含 u64 加法溢出）**/F 有条件 |
| 6 | Grok 4.5 | 有条件入账 | A 有条件/B 通过/C 有条件/D 通过/E 有条件/F 通过 |
| 7 | 豆包 | **需重做** | **A 不通过**（4 条复算：64×64 142=物理下限 142.86‰、40×40 绝对带全 PASS、SSE 不同构、广义凹形族可过）/B 有条件（h 副本/摘要自比）/C/D 有条件/**E 不通过**（P0×4+P1×4+P2×6）/F 有条件；修复清单 a-f |

**共识硬伤（已全部处置）**：①#29 Unreachable 预算漏记+让步计数污染（P0→X2 修复）；
②k/q 采样口径不对称 q>k 误差地板（P0→X1 修复）；③判定边界自指（P0→X1 ZoneBands
预注册绝对带）；④"越界判失败"口径相反（6 家→X1 binding 仅诊断）；⑤证据不同源
（→X1 SETD_COMMIT）；⑥稳态削弱（→X1 back-half 敏感性+披露）；⑦#27 不得定案
（→X5 保持候选）；⑧#28 未实装（→X5 改待实装）；⑨Q 状态互斥（→X5 钉死单一来源）；
⑩单族以偏概全（豆包→X3 p-norm p=2 并行辨识）；⑪SSE 与判据不同构（豆包→X1 相对
误差平方和）。
**终局**：整改 X1-X5 落地后 Set D 复测判定翻转（见 X4 行）；按预注册豆包 A2 路径，
**待投票确认三项**（转 C 回滚确认/pnorm-p2 族入 ADR-25 决策 1/族预选规则=物理可行域
内拟合集 SSE）。审计通过前 MFD→降级用途冻结。

## 2026-11-21 决策5 投票裁决记录（债务 #31 复判，11 家署名票）

票面：C:\Users\wenyu\Desktop\决策5.MD（归档 review\archive\）。三项裁决：

- **Q1（转 C 回滚确认）：11:0 通过**。回滚理由口径（铁律）=**原证据无效（q>k 测量缺陷+判定边界自指）**，
  不得表述为"复测已 PASS"（复测 PASS 是 Q3 规则下的反事实）；回滚≠解禁，MFD→降级保持冻结。
- **Q2（pnorm-p2 决策 1 默认族）：采纳 2 + 有条件采纳 8 + 延后 1**（Qwen 摘要级票延后，
  4 解锁条件齐备即可转确认；MiniMax"有权采纳"按有条件计）。MiMo 异议记录：pnorm 是
  "淘汰赛冠军"（拟合集 rel-SSE 44390 劣于 rational 27996、转折 189>118），胜出靠物理
  可行性非拟合优度，须原样记录。
- **Q3（预选规则=物理可行域内拟合集 rel-SSE）：11:0 通过**，防 gaming 条款=选族锁定于
  holdout 之前、一次锁定不得回退、族集合冻结扩族须重投票、绑定族不参与预选（已实装 Y1）。

共识收口条件（已按 Y1 实装/并入 Y2）：family_verdict 转折闸门+各区 n 披露、预算截断归因、
last_step_population 入 fold_digest、q≤k 下沉拟合器入口、isqrt 舍入黄金+floor 偏差披露、
拟合集逐点归档、pnorm back-half、Set E 预注册新 fixture（新种子段+第三拓扑+种子不共用）。

异议与限定保留：豆包（SET_D 带"预注册"性质属事后确立，须 Set E 重新钉；Q1↔Q3 打包生效）；
Claude（稳态 0-1/8 判据容差过紧疑噪声；膝区系统性高估须披露）；DeepSeek（单文本包缺源码
多处"未见证据"）；MiniMax（本轮=开发轮，Set E=正式确认轮；SSE 口径 rational 回退解 vs
pnorm 自由解非 like-for-like）；制度建议（Qwen/GPT-6）：不可复议结论须 ≥2 独立证据链、
统计通过与权威授权分离。

**#31 状态（决策5 裁决 + Set E 实测，2026-11-21）：转 C 撤销（回滚确认 11:0，原证据无效）。
凹形扩展（pnorm-p2）Set E 确认轮结果=2/3 拓扑 PASS、32×32 自由流 133‰ FAIL（判据 100‰，
n=3）→**未全过确认，#31 不核销**。处置候选（待负责人裁决）：①按预注册"任一 FAIL"精神
追认转 C（MFD 永久纯统计）；②按 MiniMax E 议题修微观基准语义（准入/行程时间/有向边容量）
后扩族再战再投。禁止事后调判据救场（Qwen Q3 条款）。** Set E 预注册工件=9a73ee0（新种子段
0x00C0_FFEE_3333_0000、fit/holdout 种子 +0..7/+8..15、第三拓扑 32×32、族与选族规则
继承冻结实现），取数证据=mfd-3a-sete.txt（带 SETD_COMMIT 标注）。Set E 明细：40×40
pnorm-p2 83/190/12 PASS；32×32 133/148/46 FAIL（free 超限）；64×64 48/120/20 PASS；
三拓扑预选均按 Q3 规则选中 pnorm-p2（rational 全 binding=true 被排除——修复生效）；
back-half pnorm 40×40 66/120/20 PASS。稳态诊断 0-1/8 持续注入固有（已披露）；
32×32 free 区 n=3 小样本+‰ 取整舍入敏感性（Claude/豆包精度异议）如实记录。

## 决策5 落盘执行日志（Y1–Y4，2026-11-21）

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| Y1 | 代码修复（决策5 共识收口条件）：family_verdict 合一判定核（补转折硬判据+空区 FAIL+各区 n 披露）、选族前置锁定（拟合集 rel-SSE、binding 过滤、holdout 前锁定）、q≤k 断言下沉 detailed 拟合器入口、fold_digest 补 last_step_population、isqrt 舍入黄金（含 floor 偏差披露）、预算截断归因（truncated 让步计 quota_yield_count）、mfd3a/tbench1 口径统一、back-half 补 pnorm | 已完成（2026-11-21） | fix(stage2c): Y1（359d2d8）+Y1 补遗（c0f94ba） |
| Y2 | Set E 确认轮：预注册 fixture（新种子段 0x00C0_FFEE_3333_0000、fit/holdout 种子不共用 +0..7/+8..15、第三拓扑 32x32、族与选族规则继承冻结实现）先落盘后取数 | 取数完成（2026-11-21）：40×40 PASS / 32×32 FAIL（free 133‰）/ 64×64 PASS；Q3 预选三拓扑均正确排除 binding 的 rational | feat(stage2c): Y2 预注册（9a73ee0）+取数（mfd-3a-sete.txt） |
| Y3 | 决策5 裁决入账（Q1 回滚确认 11:0/Q2 pnorm-p2 默认族 有条件/Q3 预选规则 11:0）+按 Set E 结果处置 #31 | 裁决入账完成；#31 处置=待负责人裁决（转 C 追认 / E 议题修基再战） | - |
| Y4 | issue #26 回贴（Set E 结果+处置候选）+终验 | 回贴完成（issuecomment-7045203136）；#31 处置待负责人拍板后收口 | docs(stage2c): Y2+Y3（c9f115b）+ba6d5a8 |

## 变更历史

| 日期 | 事件 |
| --- | --- |
| 2026-11-18 | 五项修订提案并入技术方案 V1.9.3；建立 ADR-NEW-14 至 ADR-NEW-18 独立文件；全部标记 Proposed |
| 2026-11-18 | 首轮评审 13 项建议裁决并落实（采纳 11 项、有条件采纳 2 项）；ADR-NEW-14/15/16/17/18 增加修订记录；主文档 V1.9.3 同步修订（草案并入，未裁决条文仍标 Proposed） |
| 2026-11-18 | 设计审计（25 条发现）与 Spike A 四件套评审完成：Spike A 证据与原型入仓（docs/benchmarks/、spikes/simscalar-spike/，cargo test 7/7 @ rustc 1.100.0）；tick 频率漂移决策数据入仓；新增 ADR-NEW-19/20 草案（状态 Proposed）；未登记决策债务入账 20 项；主文档二轮随手修（含新增 6.5 节）；裁决口径不变，全部待正式评审 |
| 2026-11-18 | Step 1（Stage 0 第零优先）：2.10 参数基线七项由项目负责人定案（参照 Cities: Skylines 2 同类数值 + Spike A / tick-rate 分析；tick 30 Hz、地图 16 km、分辨率 1 mm、速度 50 m/s、加速度 10 m/s²、经济 i64 10^12、参考场景 100k Agent、最大持续 10^9 ticks）；主文档 2.10（MD+docx）、ADR-NEW-18 定案记录同步；建立 Stage 0 工程步骤执行日志 |
| 2026-11-18 | Step 2（Spike A 收尾）：定案参数正式核对（simscalar-formal-check.py，9 项硬约束全 PASS，推荐 Q32.32）；spike crate 增 5 测试共 12 全过（dt 30 Hz 黄金值、漂移黄金、位宽、16 km/1 mm 容纳、性能冒烟 2.5 ns/op）；新增 ADR-NEW-21（SimScalar 格式定案，状态 Proposed）；2.4 加定案建议注；证据入仓 docs/benchmarks/（formal-check、simscalar-perf） |
| 2026-11-18 | Step 3（Spike B）：系统级 RNG 原型入仓 spikes/rng-spike（零依赖，9 测试全过：系统/Agent 隔离、调度无关、黄金复现、协议版本钉死、存档差异 0 vs 16 B）；算法比较（SplitMix64 0.59 ns 推荐 / xoshiro256** 0.73 / PCG64 1.14）；新增 ADR-NEW-22（RNG 协议，状态 Proposed）；EC-02 加 Spike 完成注；证据入仓 docs/benchmarks/（Spike-B-RNG-REPORT、rng-algo-comparison） |
| 2026-11-18 | Step 4（Spike C）：路网事务原子发布原型入仓 spikes/network-tx-spike（零依赖，8 测试全过：原子发布/查询版本一致/聚合规则/合法性解耦/前向进展/行走删路三方案/并发交错/性能冒烟）；新增 ADR-NEW-23（过期结果细则与行走删路定案建议，状态 Proposed）；ADR-NEW-17 待决事项全部取数；EC-01 加 Spike 完成注；证据入仓 docs/benchmarks/（Spike-C-NetworkTx-REPORT、network-tx-perf） |
| 2026-11-18 | Step 5（存档兼容验证）：save-restore-spike 入仓 spikes/save-restore-spike（零依赖，9 测试全过：EC-03 七类字段完备、tick 边界、强读档等价 N=40→M=100、弱形式、ID 高水位防复用、有状态流回读、篡改拒绝、未知 Schema 可诊断、尾字节拒绝）；读档等价测试模板入 docs/benchmarks/SaveRestore-Validation-REPORT.md；EC-03 加 Step 5 验证注；状态摘要为 FNV 占位（GlobalDigest 随 ADR-NEW-19 替换） |
| 2026-11-18 | Step 6（测试基础设施）：sim-test-support 入仓 crates/sim-test-support（零依赖，11 测试全过）——EC-07 五模块落地（fixtures 三档确定性场景/deterministic 读档等价模板 SimHarness/property 五不变量/benchmark 记录模板/assertions 状态断言）+ chaos 确定性注入钩子（忙等延迟+种子化乱序，R3 线程无关性测试口径）；crate 文档与 Cargo.toml 声明仅限 dev-dependency；EC-07 加 Step 6 验证注 |
| 2026-11-18 | Step 7（基础 CI）：.github/workflows/ci.yml 建立（R0/R1 cargo test 矩阵、EC-12 clippy -D warnings 门禁矩阵、R2 确定性场景占位）；clippy 分层实测通过（docs/benchmarks/clippy-layering-REPORT.md：HashMap/HashSet、f32/f64 含 std::primitive::f32 全路径、Instant/SystemTime 全部有效抓取，#[allow] 与按 crate 分层豁免有效；四权威 crate 全 CLEAN，49 测试回归全过）；2.9/EC-12/ADR-NEW-15 三处"可行性待验证"改为已实测 |
| 2026-11-18 | Step 8（依赖与工具链验证）：crates/dep-verify 建立（bevy "=0.18.1"/bevy_egui 0.39/rapier2d+3d 0.33）；五件套完成——依赖解析零冲突、Rust 1.89 全闭包 check（ordered-float 锁 5.4.0）、Win x86_64 build、关键 API 验证 2/2、许可证 9 项全兼容；发现 ordered-float 5.5.x 破坏 Rust 1.89 基线（须锁 ≤5.4.0 或升 1.90）；cargo-audit 发现 ruzstd 0.8.3 高危 RUSTSEC-2026-0344（基线内不可修复，安全债 #21）+ paste/ttf-parser unmaintained；rust-toolchain.toml 锁 1.100.0；证据 docs/benchmarks/dep-verify-REPORT.md；2.1 表与 6.1 表口径更新（"待复现验证"→已验证附条件） |
| 2026-11-18 | 外部评审落盘：ADR-NEW-20 新增预算让步与前向进展节（P0-01 采纳、P0-02 有条件采纳，债务 #23 立案）；ADR-NEW-19 顶层 Schema 解耦+domain separation（P0-04）；ADR-NEW-15 条 4/5 口径修订（P1-01/P1-02）；ADR-NEW-23 两阶段相交+代价更新语义（P0-06/P1-04）；外部评审裁决表 13 行入账；CI 增 MSRV 1.89 check job；issues #1-#11 回贴裁决 |
| 2026-11-18 | Step 9（workspace 稳定编译 + Stage 0 退出核对）：根 Cargo.toml workspace 建立（6 成员，统一 Cargo.lock、ordered-float 钉 5.4.0）；cargo test --workspace 51/51、cargo build、cargo +1.89 check --workspace 全闭包 exit 0；Stage 0 退出条件核对 5/6（条件 6"关键 ADR 已完成评审"为唯一硬性缺口，ADR-NEW-14~23 全部 Proposed 待裁决），记录 docs/benchmarks/Stage-0-Exit-CHECKLIST.md；Stage 0 在条件 6 完成前不宣告关闭 |
| 2026-11-19 | **正式裁决**：10 个 ADR（NEW-14~23）按五家独立 AI 评审共识完成裁决（NEW-21 采纳；NEW-14/15/16/18/19/22 有条件采纳；NEW-17/20/23 联动组定案、机制 B 定案实施前置），全部转 Accepted（附条件）；勘误 8 处（NEW-23 矛盾句、NEW-20 决策 2 末句、NEW-18 证据声明、NEW-21 Q24.40 数字、NEW-15 条 5 缺省序、NEW-17 条 8 同步、NEW-22 namespace 注、NEW-16 表增补）；主文档同步 6 处（2.9 条 5、2.8 结果字段、EC-03 表 2 行、6.5 两处、第 7 章裁决注）；机制 B 三硬约束与 ID 分配点条款入 ADR-NEW-20；新增债务 #24/#25/#26、#23 更新；Stage 0 退出条件 6 满足、Stage 0 关闭（核对更新）；issues 回贴正式裁决；第六份报告（F）修正 5 处并入（机制 B 改"默认候选待 G5 锁定"、NEW-17 条文 10 回填 MarkBlocked、NEW-19/22 条件细化、2.4 转 Accepted） |
| 2026-11-19 | **评审证据归档**：4 份独立评审报告原文（A/B/C/D）归档至 docs/architecture/review/archive/（含归档 README：报告代号映射、E 重复投递判定、F 修正采纳记录）；裁决证据链闭合 |
| 2026-11-19 | **决策投票**：状态口径=A 全票 7:0（"Accepted（附条件）"注记规范入 0.2）；Rust 基线=B 6:1（1.100.0=文档/开发/CI 主基线，1.89=MSRV 参考，msrv-check 保留）；债务 #22 关闭；条件核销清单建立（28 项）；票面归档 review/archive/决策-投票-2026-11-19.docx；issues #12/#13 关闭 |
| 2026-11-19 | **Stage 1 开工**：任务清单重建入仓（docs/architecture/Stage-1-Task-List.md，T1~T9）；债务 #19 关闭；新增 Stage 1 执行日志 |
| 2026-11-19 | **Stage 1 完成**：T1~T9 全部交付（分层骨架/SimScalar/时间命令日志/AgentId/RNG+注册表/codec+GlobalDigest/sim-save 存档/错误分层/证据表+R2 骨架），退出条件 6/6（docs/benchmarks/Stage-1-Exit-CHECKLIST.md）；条件核销清单 5 项核销（累计 8/28）；提交序列 ef6defa..fed0189 |
| 2026-11-19 | **社区门面**（运营，不涉设计决策）：README 重构+英文版 README.en.md+CONTRIBUTING.md；GitHub Topics/About 设置；good first issue 批量开设（源自条件核销清单与决策债务的可认领任务） |
| 2026-11-19 | **Stage 2A 开工**：ADR-NEW-24 草案落盘（CSR 定案、A* 确定性细则五条、机制 B 参数 g=2/L=|V|、Stage 2A 退出判据、取消/冲突语义）——收口债务 #8/#17/#18/#26 的定案部分；状态 Proposed，随实施验证后走第 7 章裁决 |
| 2026-11-19 | **Stage 2A Phase 2 实施**：sim-network（CSR 邻接稳定序、事务原子发布、代价原位、ADR-NEW-23 两阶段相交判定+合法性，9 测试）+ sim-pathfinding（ADR-NEW-24 五条确定性 A*、机制 B 预算让步 B_k=min(b·2^k,\|V\|)、解析上界断言、双计数器分离，7 测试，含 Dijkstra 最优性对拍与平局打破黄金断言）；workspace 140 测试全绿 |
| 2026-11-19 | **Stage 2A Phase 3 完成**：G5 联动组 200×200 压测（机制 A/B/C 对比）入 docs/benchmarks/（G5-stress-200x200.txt + G5-Verdict-REPORT.md）——A=7 ticks 零浪费、B=4 ticks 重复展开率 0.53（解析上界 11 实测 4 ✅）、C=100% 阻塞不可单独采用；G5 五指标全记录（最坏单 tick 耗时 0.139ms 无尖峰/重复展开率/终止率/阻塞率/假前向进展）；B 维持默认候选、A 保留切换路径（切换阈值讨论数据=0.53） |
| 2026-11-19 | **Stage 2A Phase 4 完成**：退出核对 5/5（docs/benchmarks/Stage-2A-Exit-CHECKLIST.md）；churn 补测（200×200 双向网格）重算率 0.0027 ≤ 5% PASS、阻塞率 0.00（G5-churn-200x200.txt）；事故教训=Edge 有向语义需入数据字典（债务 #8 增补）；ADR-NEW-24 裁决待第 7 章流程 |
| 2026-11-19 | **ADR-NEW-24 第二轮评审裁决**：八家独立 AI 评审 8:0 有条件采纳→Accepted（附条件）；QWEN4-MAX 两条代码缺陷修复（溢出契约 saturating、父边松弛实际边，2 回归测试）；判据口径统一（重算率分母=请求总数 3/200=0.015）；阈值过渡口径落 ADR-24（数值待 Stage 2B，债务 #27）；新增债务 #27/#28/#29/#30、#8 更新；G5 机制锁定核销（累计 9/28）；票面归档 review/archive/决策2-ADR24-投票-2026-11-19.docx；issue #24 回贴 |
| 2026-11-19 | **Stage 2B 开工（B1）**：债务 #28/#29 前置定案（tx_id 沿用原 ID 全序重排、重试上限 3 转受阻；TickBudget{remaining=Q} 全局每 tick 配额，Q 占位 65536 待 NEW-14 阈值回填）；sim-pathfinding TickBudget+request_with_budget 实装（2 新测试）；让步强读档专项 yield_state_strong_readback 通过（G3 计数入档+让步态强等价） |
| 2026-11-19 | **Stage 2B 完成**：B1~B4（#28/#29 定案实装、让步强读档专项、sim-parallel 确定性并行框架、线程无关 1/2/8+混沌注入+并发队列压测 PASS）；退出条件 4/4、硬门禁 3/3（R4 移交发布门禁），见 docs/benchmarks/Stage-2B-Exit-CHECKLIST.md 与 Stage2B-concurrent-stress.txt；条件核销（专项+压测+R3 项）；提交 ffd4516→1272e37→47ab0e2→af9427c→本次 |
| 2026-11-19 | **Stage 2C 开工（C1）**：ADR-NEW-25-MFD 交通模型与验证方案草案（模型形式候选三角基本图、MFD 默认非权威统计/降级输入、适用范围/参数校准/耦合方式三项独立验证、T-Bench-1 可复现交通基准场景、确定性判据） |
| 2026-11-19 | **Stage 2C C2**：ADR-NEW-14 预算契约细化——BudgetRegistry 声明载体定案（债务 #16 处置）、逐系统计量单位/上限候选表、L1/L2/L3 降级映射（触发=确定性预算消耗比例 70/90/100 候选）、降级状态入档入摘要；阈值数值待负责人确认 |
| 2026-11-19 | **Stage 2C C3**：C3a sim-core budget 模块（BudgetRegistry+L1/L2/L3 确定性降级状态机+canonical 编码，依赖规则 1 修订=仅基础设施）；C3b sim-traffic MFD 微观基准（GridTraffic 容量排队+定点三角基本图拟合）+T-Bench-1（40×40 密度阶梯，mfd-tbench1.txt：q 8→249‰ 呈基本图形态）；测试 12+4 全绿 |
| 2026-11-19 | **Stage 2C 完成（C4）**：六条退出条件实施面 6/6（Stage-2C-Exit-CHECKLIST）；ADR-NEW-25 Proposed 待裁决、C2 阈值候选待负责人确认；提交序列 1aeb338→6164553→0a896ae→fdd7869→本次 |
| 2026-11-19 | **决策3 投票裁决落盘**：决策 1 阈值定案回填 ADR-NEW-14（S3，提交 e12d497）；代码修复批 A/B（拟合重写、fold_digest 补字段、聚合口径逐系统、注入顺序无关、事务工作量上限，提交 3730eb0/e4bcdb4/4c64137）；ADR-NEW-25 修订转 Accepted（附条件）（S4）；10 票裁决记录+2 份 ADR-24 补充入账（S5） |
| 2026-11-19 | **V1 3a 独立参照验证**：三集协议（A 拟合/B 试点/C 确认）+多 seed 平均、evaluate_fit 三区评估 API、判据复核回填 ADR-NEW-25；确认集饱和 102‰ PASS、自由流 190‰ FAIL→模型形式凹形缺陷立债务 #31（裁决前 MFD 仅统计展示）；docs/benchmarks/mfd-3a-reference.txt |
| 2026-11-19 | **V2 3c 生产代码路径**：需求压力降级通道（sim-traffic demand_pressure_bp→BudgetRegistry::note_demand_pressure，effective=max、需求单独触发封顶 L1、状态入档入摘要）+ADR-NEW-26 草案；专用耦合线程无关测试 mfd_coupling（1/2/8 线程+混沌注入+串行参考三路摘要一致）；budget+traffic 测试 12+14 全绿 |
| 2026-11-19 | **V3 债务 #30 A* 细则补遗**：astar_edge_cases 5 测试（reopen 差分=违反代价下限前提场景 13 vs 1010、等代价前驱=先发现保留、乱序注入=代价/摘要稳定、CSR u32=179,400 边 300×300 网格、h 一致性不变式边界书面化） |
| 2026-11-19 | **V4 债务 #27 阈值基线**：G5 三场景（open/obstacle-20%/maze）各 200 请求实测——dup_rate 证伪为切换判据（max 0.664≈2/3 结构上界）；候选判据=单次尝试展开>4096 或让步≥8 切机制 A（确定性工作量单位，墙钟仅遥测）；数据 docs/benchmarks/G5-threshold-baseline.txt，ADR-NEW-24 阈值段回填 |
| 2026-11-19 | **V5 收官**：条件核销 #30（A* 细则补遗 5 测试）、#27 基线进展注（数值定案随 G2）；ADR-NEW-25 转 Verified 条件进展表（3a 部分通过+债务 #31、3c 半达成、阈值候选、R3 过/R4 移交）；goal-53eb075f V1–V5 全程完成 |
| 2026-11-19 | **决策4 投票裁决落盘（债务 #31）**：10 票 A7:B2:C1→凹形扩展启动；ADR-NEW-25 决策 1/3a 判据修订（函数族写死+去自指+Set D）、ADR-NEW-26 k_ref 特征密度修订、代码整改 6 项（判据常量单一来源/门禁对齐/fail-safe 改向/双实现删除/聚合断言）；MiniMax E 异议与 Claude 疑云如实记录 |
| 2026-11-20 | **债务 #31 执行面 W1–W3**：凹形有理饱和拟合器实装（自由估计+越界披露+物理可行回退）、Set D 复测（两拓扑自由流 148/142‰ FAIL）→按预注册终止条件**转 C**：MFD 永久纯统计展示、ADR-NEW-26 通道冻结、债务 #31 核销；E 议题（微观基准修基）为唯一重开路径 |
| 2026-11-20 | **债务闭合审计与整改**：7 家审计（审计1.MD）确认转 C 程序忠实但证据基础四项 P0→债务 #31 回退"暂定（审计退回）"；整改 X1 P0 修复（k/q 口径、预注册绝对带、越界判失败、目标同构，c8a7a28）、X2 预算修复（Unreachable 入账、让步计数分离）；审计包/单文本包/分享规矩（d8f1a11/eaf7b1b/c2b2e53），issue #27 |
| 2026-11-20 | **债务闭合审计整改 X3–X5**：X3 p-norm p=2 族并行辨识（evaluate_q_fn 单核防双实现、拟合集 rel-SSE 预选防 gaming，ce4a705）；X4 Set D 复测**判定翻转：转 C 不成立**（pnorm-p2 物理可行两拓扑三区全 PASS，证据 commit ce4a705，125d71d）；X5 状态/文档卫生（#27 保持候选、#28 改待实装、Q=65536 已定案单一来源 TICK_QUOTA、README/主文档/核销清单/两 CHECKLIST 口径统一、budget last_demand_bp 归档修复） |

## 关联

- 现行主文档：docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md（正式版，Markdown）；评审流转副本为工作区根目录 CitySim-rs技术方案V1.9.3.docx。
- 历史版本与回退依据：docs/architecture/history/（CitySim-rs V1.9.2 技术方案、CitySim-rs V1.9.2 修订提案（五项）原件、CitySim-rs-ADR文档包（修订前）.zip）。
- 历史决策（ADR-NEW-01 至 ADR-NEW-13 及原有 ADR）索引见主文档第 6 章。ADR-NEW-01 至 ADR-NEW-13 独立建档排期：V1.9.3 正式版发布前补齐，与 ADR-NEW-14 至 20 保持同等建档粒度（评审建议 13）。
- 证据归档：docs/benchmarks/——analysis.py（复核重建版）、analysis-output.txt、tick-rate-drift.py、tick-rate-drift-output.txt、Spike-A-SimScalar-REPORT.md（修订版）。
- Spike A 原型：spikes/simscalar-spike/（SimScalar Q32.32 定点原型 + 7 项黄金测试；rustc 1.100.0 实测 7/7，文档基线 1.89 待收敛——见二轮 P7）。


---

## FILE: MANIFEST.txt

```
CitySim-rs-MFD31D-REVIEW.zip — MANIFEST
基准提交: 1d8b7792e8fbadf10173fbfcd8df6fc5d4a3b998（2026-11-21）
口径说明:
1. mfd-3a-sete.txt 由预注册 fixture 9a73ee0 生成，文件内 SETD_COMMIT 行标注取数提交。
2. Set E 结果：40x40 PASS / 32x32 FAIL（free 133‰，判据 100‰）/ 64x64 PASS；
   32x32 back-half 同值 133‰（前后半窗口一致=稳定信号非瞬态）。
3. free 区 n=3（32x32），样本数小；‰ 取整舍入影响量级约 ±50‰（q 分辨率），如实披露。
4. Q3 预选规则=物理可行域内拟合集 rel-SSE；三拓扑均选中 pnorm-p2（rational 全 binding 被排除）。
5. 口径铁律：转 C 撤销的理由=原证据无效（q>k 测量缺陷+边界自指），非"复测已 PASS"；
   回滚≠解禁，MFD→降级通道冻结。
6. 裁决=债务 #31 终局处置二选一（①追认转 C / ②MiniMax E 议题修基再战）。

```
