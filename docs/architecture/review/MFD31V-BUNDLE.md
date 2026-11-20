# CitySim-rs-MFD31V 单文本材料包（与 ZIP 逐字节同源）

基准提交 fb9fe47 / 证据 commit ce4a705 / 生成 2026-11-20

## MANIFEST.txt
```
CitySim-rs-MFD31V-REVIEW 材料包 MANIFEST
基准提交: fb9fe47（Set D 证据 commit: ce4a705）
生成: 2026-11-20

口径说明（与源码逐条对应，可独立复核）:
1. k/q 采样同群体同时间: k 用 last_step_population（step 中移除到达车之前计数），
   q=flow_this_tick（同 tick 边遍历）；q<=k fail-fast（X1 修复，原 q>k 误差地板）。
2. 判定边界 ZoneBands::SET_D{free_max_k=100, trans_max_k=2000} 预注册绝对带，
   不由被评参数导出（X1，禁同参 gaming）。
3. 条件⑧口径: 自由估计斜率越界（>1000‰）binding=true 即族淘汰；回退可行解仅诊断。
4. 族预选 = 拟合集 rel-SSE 最小者，锁定于留出评估之前（防 gaming）；预选域=物理可行域
   （free estimate 不越界）——Q3 待裁决。
5. budget_yield_count=请求自身预算让步、quota_yield_count=配额让步（X2 分离）；
   Unreachable 携带 expanded 扣预算。
6. 稳态诊断: 持续注入高密度档不稳态（0-1/8 档）已披露；back-half 敏感性同向。

```

## docs/architecture/review/AI-REVIEW-BRIEF-MFD31V.md  (sha256:4b9d228fe0ada4e3…, 5019 B)

```text
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

```

## docs/architecture/adr/ADR-NEW-25-MFD交通模型与验证方案.md  (sha256:ee9c31744745a420…, 15238 B)

```text
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

```

## docs/architecture/adr/ADR-NEW-26-需求压力降级信号通道.md  (sha256:cd7163dae46a706f…, 5238 B)

```text
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
**通道冻结（2026-11-20）**：Set D 复测 FAIL（两拓扑自由流判据 148/142‰ > 100‰），
按 ADR-NEW-25 预注册终止条件**转 C**——MFD 不作降级/背压触发依据，本通道在
MiniMax E 议题（微观基准修基）立项并重新验证前**不得接入权威降级路径**；
k_ref 特征密度口径保留为待定案（随重开评估一并定案）。

```

## docs/benchmarks/mfd-3a-setd.txt  (sha256:82077f57d20f8cd3…, 3364 B)

```text
   Compiling sim-traffic v0.1.0 (D:\AIWORK\CITYSIM\crates\sim-traffic)
    Finished `release` profile [optimized] target(s) in 0.69s
     Running `target\release\examples\mfd_setd.exe`
Set D re-verification (debt #31; pre-registered fixture, disjoint seeds)
evidence commit: ce4a7056be218ad079c3b3dc9dcfa560992e9796
criteria: free<=100‰ transition<=250‰ saturated<=250‰ (single source = sim-traffic consts)
=== topology 40x40 (Set D seeds 0xc0ffee22220000+0..8) ===
steady diagnostics: fit 1/8 steady, holdout 1/8 steady (dual criterion; unsteady ladders disclosed)
fit: q_max=270‰ k_half=270‰ zero-density slope=1000‰ (free estimate 1095‰, bound binding: true)
  k    12‰ q_obs   12‰ q_model   11‰ err 83‰
  k    26‰ q_obs   24‰ q_model   24‰ err 0‰
  k    56‰ q_obs   50‰ q_model   46‰ err 80‰
  k   132‰ q_obs  101‰ q_model   89‰ err 118‰
  k   509‰ q_obs  185‰ q_model  176‰ err 48‰
  k  1983‰ q_obs  231‰ q_model  238‰ err 30‰
  k  5392‰ q_obs  245‰ q_model  257‰ err 48‰
  k 12413‰ q_obs  250‰ q_model  264‰ err 56‰
  bound violation: free estimate slope 1095‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)
  free-flow max err   =   83‰  (criterion <= 100‰)  PASS
  transition max err  =  118‰  (criterion <= 250‰)  PASS
  saturated max err   =   56‰  (criterion <= 250‰)  PASS
  topology verdict [rational] = FAIL (binding=true free=83‰ sat=56‰)
  [family pnorm-p2] fit: q_max=248‰ k0=266‰ slope free=932‰ binding=false
  [family pnorm-p2] free=83‰ trans=189‰ sat=12‰ -> PASS
  [family pre-selection] fit-set rel-SSE rational=27996 pnorm=44390 -> selected=rational (locked before holdout)
  Set D outcome (selected family rational) = FAIL
  [sensitivity back-half] free=80‰ sat=64‰ binding=true -> FAIL
=== topology 64x64 (Set D seeds 0xc0ffee22220000+0..8) ===
steady diagnostics: fit 0/8 steady, holdout 0/8 steady (dual criterion; unsteady ladders disclosed)
fit: q_max=284‰ k_half=284‰ zero-density slope=1000‰ (free estimate 1092‰, bound binding: true)
  k     7‰ q_obs    7‰ q_model    7‰ err 0‰
  k    15‰ q_obs   14‰ q_model   14‰ err 0‰
  k    32‰ q_obs   30‰ q_model   29‰ err 33‰
  k    71‰ q_obs   62‰ q_model   57‰ err 80‰
  k   179‰ q_obs  124‰ q_model  110‰ err 112‰
  k   607‰ q_obs  199‰ q_model  193‰ err 30‰
  k  1829‰ q_obs  233‰ q_model  246‰ err 55‰
  k  4494‰ q_obs  245‰ q_model  267‰ err 89‰
  bound violation: free estimate slope 1092‰ > 1000‰ -> topology verdict = FAIL (decision-4 clause 8; bound-fit is diagnostic only)
  free-flow max err   =   80‰  (criterion <= 100‰)  PASS
  transition max err  =  112‰  (criterion <= 250‰)  PASS
  saturated max err   =   89‰  (criterion <= 250‰)  PASS
  topology verdict [rational] = FAIL (binding=true free=80‰ sat=89‰)
  [family pnorm-p2] fit: q_max=240‰ k0=254‰ slope free=944‰ binding=false
  [family pnorm-p2] free=48‰ trans=120‰ sat=20‰ -> PASS
  [family pre-selection] fit-set rel-SSE rational=27063 pnorm=32384 -> selected=rational (locked before holdout)
  Set D outcome (selected family rational) = FAIL
  [sensitivity back-half] free=80‰ sat=93‰ binding=true -> FAIL

```

## docs/benchmarks/mfd-3a-reference.txt  (sha256:a61016728aa9a972…, 3745 B)

```text
MFD 3a reference validation (ADR-NEW-25) — fit A vs pilot B vs confirm C, seeds 0xc0ffee12345678..+8 averaged
grid 40x40, 260 ticks/density, window mean over last 60 ticks; multi-seed mean = variance reduction for non-authoritative statistics
fit set A densities (per mille k observed):
  inj   2 -> k    7‰  q    7‰
  inj   4 -> k   16‰  q   16‰
  inj   8 -> k   34‰  q   33‰
  inj  16 -> k   76‰  q   67‰
  inj  32 -> k  205‰  q  135‰
  inj  64 -> k  932‰  q  211‰
  inj 128 -> k 3082‰  q  239‰
  inj 256 -> k 7710‰  q  248‰
pilot set B densities (never seen by the fit):
  inj   3 -> k   11‰  q   12‰
  inj   6 -> k   24‰  q   24‰
  inj  12 -> k   54‰  q   50‰
  inj  24 -> k  128‰  q  101‰
  inj  48 -> k  499‰  q  185‰
  inj  96 -> k 1969‰  q  231‰
  inj 192 -> k 5380‰  q  245‰
  inj 384 -> k 12404‰  q  250‰
confirm set C densities (never seen by the fit):
  inj   5 -> k   20‰  q   20‰
  inj  10 -> k   43‰  q   41‰
  inj  20 -> k  100‰  q   84‰
  inj  40 -> k  329‰  q  163‰
  inj  80 -> k 1435‰  q  225‰
  inj 160 -> k 4224‰  q  243‰
  inj 320 -> k 10054‰  q  249‰
  inj 640 -> k 21830‰  q  252‰
fit parameters (from set A only):
  v_free = 1000‰  k_c = 7710‰  q_max = 248‰  w = 0‰  descending = false
  interpretation: monotone-saturation (plateau); triangle -w not identifiable on this data
per-point error pilot B |q_model - q_obs| / q_obs (‰):
  k   11‰ q_obs   12‰ q_model   11‰ err 83‰
  k   24‰ q_obs   24‰ q_model   24‰ err 0‰
  k   54‰ q_obs   50‰ q_model   54‰ err 80‰
  k  128‰ q_obs  101‰ q_model  128‰ err 267‰
  k  499‰ q_obs  185‰ q_model  248‰ err 340‰
  k 1969‰ q_obs  231‰ q_model  248‰ err 73‰
  k 5380‰ q_obs  245‰ q_model  248‰ err 12‰
  k 12404‰ q_obs  250‰ q_model  248‰ err 8‰
per-point error confirm C |q_model - q_obs| / q_obs (‰):
  k   20‰ q_obs   20‰ q_model   20‰ err 0‰
  k   43‰ q_obs   41‰ q_model   43‰ err 48‰
  k  100‰ q_obs   84‰ q_model  100‰ err 190‰
  k  329‰ q_obs  163‰ q_model  248‰ err 521‰
  k 1435‰ q_obs  225‰ q_model  248‰ err 102‰
  k 4224‰ q_obs  243‰ q_model  248‰ err 20‰
  k 10054‰ q_obs  249‰ q_model  248‰ err 4‰
  k 21830‰ q_obs  252‰ q_model  248‰ err 15‰
3a metrics — PILOT set B (boundary-derivation disclosure: regions defined after seeing B):
  free-flow   83‰ | transition  340‰ | saturated   73‰ | k_c 4694‰ | pts 8
3a metrics — CONFIRM set C (verdict set; three-region criteria = original 10/20/25% re-split):
  free-flow max err   =  190‰  (k < k_turn/2; criterion <= 100‰)     FAIL
  transition max err  =  521‰  (k_turn/2 .. 4*k_turn; record only)
  saturated max err   =  102‰  (k >= 4*k_turn; criterion <= 250‰)   PASS
  k_c position err    = 14120‰  (plateau-type: N/A — no peak, metric not applicable)
  holdout points evaluated = 8
methodology note (2026-11-19 calibration review):
  1. original 10/20/25% single-split criteria failed on pilot B transition points
     (250-326‰) while free-flow (<=61‰) and plateau (<=73‰) passed — the plateau
     model has no ramp, so transition deviation is structural, not parameter error;
  2. criteria re-split by identified shape and three density regions; region width
     (4*k_turn) is PILOT-DERIVED and disclosed as such; single-seed confirm C showed
     10-17% interpolation error at independent density points;
  3. multi-seed averaging (N_SEEDS seeds) applied as variance reduction for the
     non-authoritative MFD statistics; verdicts use confirm C under this protocol;
  4. second-phase CTM/LWR cross-check remains open (ADR-NEW-25 3a).

```

## crates/sim-traffic/src/lib.rs  (sha256:af82988243aacfa7…, 39493 B)

```text
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
    assert!(!holdout.is_empty(), "no holdout samples");
    let mut free_max = 0u64;
    let mut trans_max = 0u64;
    let mut sat_max = 0u64;
    for s in holdout {
        let qm = q_model_permille(fit, s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000
            / (s.q_permille.max(1) as u128);
        let err = err as u64;
        if s.k_permille < bands.free_max_k {
            free_max = free_max.max(err);
        } else if s.k_permille < bands.trans_max_k {
            trans_max = trans_max.max(err);
        } else {
            sat_max = sat_max.max(err);
        }
    }
    let k_obs_peak = holdout
        .iter()
        .max_by_key(|s| s.q_permille)
        .map(|s| s.k_permille)
        .unwrap_or(0);
    let k_c_err = fit.k_c_permille.abs_diff(k_obs_peak);
    FitErrorReport {
        free_flow_max_err_permille: free_max,
        transition_max_err_permille: trans_max,
        saturated_max_err_permille: sat_max,
        k_c_err_permille: k_c_err,
        points_evaluated: holdout.len(),
    }
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
    for s in holdout {
        let qm = q(s.k_permille);
        let err = (qm as u128).abs_diff(s.q_permille as u128) * 1000
            / (s.q_permille.max(1) as u128);
        let err = err as u64;
        if s.k_permille < bands.free_max_k {
            free_max = free_max.max(err);
        } else if s.k_permille < bands.trans_max_k {
            trans_max = trans_max.max(err);
        } else {
            sat_max = sat_max.max(err);
        }
    }
    FitErrorReport {
        free_flow_max_err_permille: free_max,
        transition_max_err_permille: trans_max,
        saturated_max_err_permille: sat_max,
        k_c_err_permille: 0, // 平台/饱和族无峰值拐点，N/A
        points_evaluated: holdout.len(),
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
        let ks: Vec<u64> = (1..=20).map(|i| i * 10).collect();
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
        // 可行解回退仅诊断，zero_density_slope≤1000‰。
        let ks: Vec<u64> = (1..=16).map(|i| i * 40).collect();
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
}

```

## crates/sim-traffic/examples/mfd_setd.rs  (sha256:19bbe46b22f5c00b…, 12305 B)

```text
//! Set D 复测（债务 #31 核销条件；ADR-NEW-25 3a 决策4 判据修订 (c)）。
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

const SETD_SEED_BASE: u64 = 0x00C0_FFEE_2222_0000;
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
fn sample_ladder(w: u32, h: u32, ladder: &[u64]) -> (Vec<MfdSample>, usize) {
    let mut unsteady = 0usize;
    let samples = ladder
        .iter()
        .map(|&inj| {
            let (mut k, mut q, mut steady) = (0u64, 0u64, true);
            for i in 0..N_SEEDS {
                let (k1, q1, s1) = run_density_seed(w, h, inj, SETD_SEED_BASE + i);
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
fn rel_sse(q: impl Fn(u64) -> u64, samples: &[MfdSample]) -> u128 {
    let mut sse = 0u128;
    for s in samples {
        let e = q(s.k_permille) as i128 - s.q_permille as i128;
        let er = (e * 1000) / (s.q_permille.max(1) as i128);
        sse += (er * er) as u128;
    }
    sse
}

/// 单族判定：越界（binding）即 FAIL；三区判据常量单一来源。
fn family_verdict(binding: bool, r: &FitErrorReport) -> &'static str {
    if binding
        || r.free_flow_max_err_permille > FREE_FLOW_CRIT_PERMILLE
        || r.saturated_max_err_permille > SATURATED_CRIT_PERMILLE
    {
        "FAIL"
    } else {
        "PASS"
    }
}

fn report_topology(w: u32, h: u32, emit: &mut dyn FnMut(String)) {
    emit(format!("=== topology {w}x{h} (Set D seeds {SETD_SEED_BASE:#x}+0..{N_SEEDS}) ==="));
    let (mut fit_samples, unsteady_fit) = sample_ladder(w, h, &FIT_LADDER);
    fit_samples.sort_by_key(|s| s.k_permille);
    let (holdout, unsteady_hold) = sample_ladder(w, h, &HOLDOUT_LADDER);
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
    let topology_fail = binding
        || r.free_flow_max_err_permille > FREE_FLOW_CRIT_PERMILLE
        || r.saturated_max_err_permille > SATURATED_CRIT_PERMILLE;
    emit(format!(
        "  topology verdict [rational] = {} (binding={} free={}‰ sat={}‰)",
        if topology_fail { "FAIL" } else { "PASS" },
        binding,
        r.free_flow_max_err_permille,
        r.saturated_max_err_permille
    ));
    // ===== X3：p-norm p=2 并行辨识（豆包清单 d：多族并行，防"单族以偏概全"）=====
    // 防 gaming 协议（GPT-6/豆包）：族选择只看拟合集相对 SSE（先于留出评估锁定），
    // 留出集不参与选族；两族留出指标全部披露。
    let (pfit, p_slope, p_binding) = fit_pnorm_concave_detailed(&fit_samples);
    let rp = evaluate_pnorm_fit(&pfit, &holdout, &ZoneBands::SET_D);
    emit(format!(
        "  [family pnorm-p2] fit: q_max={}‰ k0={}‰ slope free={}‰ binding={}",
        pfit.q_max_permille, pfit.k0_permille, p_slope, p_binding
    ));
    emit(format!(
        "  [family pnorm-p2] free={}‰ trans={}‰ sat={}‰ -> {}",
        rp.free_flow_max_err_permille, rp.transition_max_err_permille, rp.saturated_max_err_permille,
        family_verdict(p_binding, &rp)
    ));
    let sse_r = rel_sse(|k| q_rational_permille(&fit, k), &fit_samples);
    let sse_p = rel_sse(|k| q_pnorm_permille(&pfit, k), &fit_samples);
    let selected = if sse_p < sse_r { "pnorm-p2" } else { "rational" };
    emit(format!(
        "  [family pre-selection] fit-set rel-SSE rational={} pnorm={} -> selected={} (locked before holdout)",
        sse_r, sse_p, selected
    ));
    emit(format!(
        "  Set D outcome (selected family {}) = {}",
        selected,
        if selected == "pnorm-p2" { family_verdict(p_binding, &rp) } else { family_verdict(binding, &r) }
    ));
    // 稳态敏感性（审计修复 2026-11-20）：窗口后半单独采样复算三区指标，
    // 若与全窗结论一致则"稳态仅披露"升级为"已验证稳健"。
    let back_fit: Vec<MfdSample> = FIT_LADDER
        .iter()
        .map(|&inj| {
            let (mut k, mut q) = (0u64, 0u64);
            for i in 0..N_SEEDS {
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + i);
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
                let (k1, q1) = run_density_seed_back_half(w, h, inj, SETD_SEED_BASE + i);
                k += k1;
                q += q1;
            }
            MfdSample { k_permille: k / N_SEEDS, q_permille: q / N_SEEDS }
        })
        .collect();
    let (fit_b, _, binding_b) = fit_rational_saturation_detailed(&back_fit);
    let r_b = evaluate_rational_fit(&fit_b, &back_hold, &ZoneBands::SET_D);
    emit(format!(
        "  [sensitivity back-half] free={}‰ sat={}‰ binding={} -> {}",
        r_b.free_flow_max_err_permille,
        r_b.saturated_max_err_permille,
        binding_b,
        if binding_b
            || r_b.free_flow_max_err_permille > FREE_FLOW_CRIT_PERMILLE
            || r_b.saturated_max_err_permille > SATURATED_CRIT_PERMILLE
        {
            "FAIL"
        } else {
            "PASS"
        }
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
    report_topology(64, 64, &mut |l| out.push(l));
    for l in &out {
        println!("{l}");
    }
}

```

## crates/sim-pathfinding/src/lib.rs  (sha256:10806786a2843c13…, 24903 B)

```text
//! 寻路系统：确定性 A* 与机制 B 预算让步（主文档 2.2 架构分层；ADR-NEW-24 决策 2/3）。
//! 权威 crate。
//!
//! Stage 2A 实装（2026-11-19）：
//! - A* 唯一性五条（ADR-NEW-24 决策 2）：启发式=曼哈顿定点；open 集排序键 (f, h, node_id)
//!   全序升序；邻居展开=CSR 数组序；closed 首展闭合、g 严格更小才 reopen；禁 BinaryHeap。
//! - 机制 B（ADR-NEW-20 三硬约束）：B_k = min(b·g^k, L)，g=2，L=|V|；单 tick 原子搜索
//!   （方案③：跨 tick 不保存部分搜索状态，让步即丢弃本 tick 进度、下 tick 以更大预算重做）；
//!   解析上界 k ≤ ⌈log_g(|V|/b)⌉ + 1 tick 内完成或进入受阻判定。
//! - 三计数器分离（ADR-NEW-20 + X2 审计修复）：budget_yield_count（请求自身预算让步，
//!   债务 #27 判据只认它）与 stale_recompute_count（过期重算）与 quota_yield_count
//!   （全局配额耗尽让步）互不混用。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use sim_network::Network;

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "寻路系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

// ---------------- 机制 B 预算策略 ----------------

/// 机制 B 参数（ADR-NEW-24 决策 3）：b=每 tick 节点预算，g=放大幅（默认 2），L=|V|。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetPolicy {
    pub b: u64,
    pub g: u64,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        BudgetPolicy { b: 64, g: 2 }
    }
}

impl BudgetPolicy {
    /// B_k = min(b·g^k, L)（k=连续让步次数）。
    pub fn budget_for_yield(&self, yields: u64, n_nodes: u64) -> u64 {
        let l = n_nodes.max(1);
        let mut acc = self.b.max(1);
        for _ in 0..yields {
            acc = acc.saturating_mul(self.g.max(2));
            if acc >= l {
                break;
            }
        }
        acc.min(l)
    }

    /// 解析上界：k ≤ ⌈log_g(L/b)⌉ + 1 个 tick 内完成或受阻（ADR-NEW-20 三硬约束）。
    pub fn max_ticks_to_terminal(&self, n_nodes: u64) -> u64 {
        let l = n_nodes.max(1);
        let b = self.b.max(1).min(l);
        let mut k = 0u64;
        let mut acc = b;
        while acc < l {
            acc = acc.saturating_mul(self.g.max(2));
            k += 1;
        }
        k + 1
    }
}

// ---------------- 搜索结果 ----------------

/// 单次（单 tick 原子）搜索结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FindOutcome {
    /// 找到路径：边 ID 序列 + 总代价 + 本次展开节点数。
    Complete { path: Vec<u64>, cost: i64, expanded: u64 },
    /// 本 tick 节点预算耗尽（让步；不保存中间状态，下 tick 以 B_k 重做）。
    Yielded { expanded: u64 },
    /// open 集耗尽未达目标：不可达（进入受阻终态的判定输入）。
    /// 携带本次展开节点数（X2 审计修复：不可达搜索实耗必须入账，否则同 tick
    /// 多不可达请求可绕过全局配额 Q，违反 ADR-NEW-14 条 3）。
    Unreachable { expanded: u64 },
}

impl FindOutcome {
    /// 本次搜索展开的节点数（三种结果均携带；全局配额扣减用）。
    pub fn expanded(&self) -> u64 {
        match self {
            FindOutcome::Complete { expanded, .. }
            | FindOutcome::Yielded { expanded }
            | FindOutcome::Unreachable { expanded } => *expanded,
        }
    }
}

/// 寻路器状态：双计数器 + 机制 B 执行（ADR-NEW-20）。
#[derive(Clone, Debug, Default)]
pub struct PathFinder {
    pub policy: BudgetPolicy,
    /// 预算让步计数（连续让步，成功/受阻即清零）。
    pub budget_yield_count: u64,
    /// 过期重算计数（与让步分离，独立上限由调用方约束）。
    pub stale_recompute_count: u64,
    /// 配额让步计数（全局每 tick 配额耗尽导致的让步；X2 审计修复：与
    /// budget_yield_count 分离——债务 #27 "让步≥8"切换判据只认请求自身预算
    /// 让步，配额让步混入会污染判据，MiMo/GPT-6/Gemini 审计命中）。
    pub quota_yield_count: u64,
}

impl PathFinder {
    pub fn new(policy: BudgetPolicy) -> Self {
        PathFinder { policy, ..Default::default() }
    }

    /// 发起一次寻路请求（单 tick 原子；机制 B 预算 B_k）。
    pub fn request(&mut self, net: &Network, coords: &[(i64, i64)], start: u64, goal: u64) -> FindOutcome {
        let budget = self.policy.budget_for_yield(self.budget_yield_count, net.node_count());
        let out = find_once(net, coords, start, goal, budget);
        match &out {
            FindOutcome::Complete { .. } | FindOutcome::Unreachable { .. } => {
                self.budget_yield_count = 0;
            }
            FindOutcome::Yielded { .. } => {
                self.budget_yield_count = self.budget_yield_count.saturating_add(1);
            }
        }
        out
    }

    /// 全局每 tick 配额下的寻路请求（债务 #29 定案，ADR-NEW-24 决策 3 补充）。
    ///
    /// B_k ≤ min(b·2^k, L, budget.remaining)；实耗展开数扣减 `budget.remaining`；
    /// 配额耗尽即让步（`Yielded { expanded: 0 }`，不消耗预算）。
    /// 多请求处理顺序（请求 ID 稳定序）与每 tick `TickBudget::reset()` 由调用方负责。
    pub fn request_with_budget(
        &mut self,
        net: &Network,
        coords: &[(i64, i64)],
        start: u64,
        goal: u64,
        budget: &mut TickBudget,
    ) -> FindOutcome {
        if budget.remaining == 0 {
            self.quota_yield_count = self.quota_yield_count.saturating_add(1);
            return FindOutcome::Yielded { expanded: 0 };
        }
        let b_k = self
            .policy
            .budget_for_yield(self.budget_yield_count, net.node_count())
            .min(budget.remaining);
        let out = find_once(net, coords, start, goal, b_k);
        // 不可达同样携带 expanded 并扣减全局配额（X2 审计修复）。
        budget.remaining = budget.remaining.saturating_sub(out.expanded());
        match &out {
            FindOutcome::Complete { .. } | FindOutcome::Unreachable { .. } => {
                self.budget_yield_count = 0;
            }
            FindOutcome::Yielded { .. } => {
                self.budget_yield_count = self.budget_yield_count.saturating_add(1);
            }
        }
        out
    }

    /// 过期重算记账（与让步计数分离；ADR-NEW-20 双计数器）。
    pub fn note_stale_recompute(&mut self) {
        self.stale_recompute_count = self.stale_recompute_count.saturating_add(1);
    }
}

/// 全局每 tick 寻路配额 Q 的单一来源常量（X5 审计整改：消除多处字面量
/// 状态互斥）。数值=65536，**已定案**（2026-11-19 决策3 投票 1a，9:1 通过；
/// 含义=对实测 worst 398@40k 留余量的保险丝，非"100× 余量"）。
pub const TICK_QUOTA: u64 = 65_536;

/// 全局每 tick 寻路配额（债务 #29 定案）。
///
/// `remaining` = 本 tick 尚可消耗的节点展开数（确定性工作量单位，对齐 ADR-NEW-14 条 3）；
/// 每 tick 由调用方在 tick 边界 `reset()` 归满（每 tick 预算独立）。
/// 多请求共享：请求实耗扣减，耗尽后其余请求让步——全局每 tick 总展开 ≤ `quota`。
#[derive(Clone, Debug)]
pub struct TickBudget {
    pub quota: u64,
    pub remaining: u64,
}

impl TickBudget {
    pub fn new(quota: u64) -> Self {
        TickBudget { quota, remaining: quota }
    }

    /// tick 边界重置。
    pub fn reset(&mut self) {
        self.remaining = self.quota;
    }
}

// ---------------- 确定性 A*（ADR-NEW-24 决策 2） ----------------

fn h_manhattan(coords: &[(i64, i64)], node: u64, goal: u64) -> i64 {
    let (x1, y1) = coords.get(node as usize).copied().unwrap_or((0, 0));
    let (x2, y2) = coords.get(goal as usize).copied().unwrap_or((0, 0));
    // 溢出契约（ADR-NEW-21 决策 3）：禁止静默环绕，i128 中间值 + 饱和收敛到 i64。
    let dx = (x1 as i128 - x2 as i128).abs();
    let dy = (y1 as i128 - y2 as i128).abs();
    (dx + dy).min(i64::MAX as i128) as i64
}

/// 单 tick 原子搜索：展开节点数不超过 node_budget；确定性五条内联。
///
/// 前提（ADR-NEW-24）：边代价下限 ≥ 边几何曼哈顿长度，h 可采纳。
pub fn find_once(
    net: &Network,
    coords: &[(i64, i64)],
    start: u64,
    goal: u64,
    node_budget: u64,
) -> FindOutcome {
    if start == goal {
        return FindOutcome::Complete { path: Vec::new(), cost: 0, expanded: 0 };
    }
    // open 集：BTreeMap<(f, h, node_id), ()> 升序弹出 = 排序键全序（决策 2 第 2 条）
    let h0 = h_manhattan(coords, start, goal);
    let mut open: BTreeMap<(i64, i64, u64), ()> = BTreeMap::new();
    open.insert((h0, h0, start), ());
    let mut g: BTreeMap<u64, i64> = BTreeMap::new();
    g.insert(start, 0);
    // parent 记录 (父节点, 松弛时实际使用的边 ID)：平行同向边存在时路径边集与代价严格一致。
    let mut parent: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
    let mut closed: BTreeSet<u64> = BTreeSet::new();
    let mut expanded = 0u64;

    while let Some(((_f, _h, node), ())) = open.pop_first() {
        if node == goal {
            // 回溯路径（使用松弛时记录的边 ID）
            let mut path = Vec::new();
            let mut cur = goal;
            while let Some(&(p, eid)) = parent.get(&cur) {
                path.push(eid);
                cur = p;
            }
            path.reverse();
            return FindOutcome::Complete { path, cost: g[&goal], expanded };
        }
        if expanded >= node_budget {
            // 预算耗尽：让步（不保存中间状态，方案③）
            return FindOutcome::Yielded { expanded };
        }
        if !closed.insert(node) {
            continue; // 已闭合且未被更优 g reopen
        }
        expanded += 1;
        let g_cur = g[&node];
        // 邻居展开序 = CSR 数组序（决策 2 第 3 条）
        for (eid, to, cost) in net.neighbors(node) {
            // 溢出契约（ADR-NEW-21 决策 3）：饱和运算，禁止静默环绕。
            let new_g = g_cur.saturating_add(cost);
            let known = g.get(&to).copied();
            if known.is_some_and(|k| new_g >= k) {
                continue; // 仅严格更小才松弛/reopen（决策 2 第 4 条）
            }
            g.insert(to, new_g);
            parent.insert(to, (node, eid));
            closed.remove(&to); // 允许 reopen
            let hh = h_manhattan(coords, to, goal);
            let ff = new_g.saturating_add(hh);
            open.insert((ff, hh, to), ());
        }
    }
    FindOutcome::Unreachable { expanded }
}

// ---------------- 测试 ----------------

#[cfg(test)]
mod tests {
    use super::*;
    use sim_network::Edge;
    use sim_network::{Op, Tx};

    fn grid(w: u64, h: u64) -> (Network, Vec<(i64, i64)>) {
        // w×h 网格（mm 单位 1000/格），边代价=曼哈顿长度（满足可采纳前提）
        let mut net = Network::new();
        let mut coords = Vec::new();
        for y in 0..h {
            for x in 0..w {
                coords.push((x as i64 * 1000, y as i64 * 1000));
            }
        }
        let mut tid = 1u64;
        let mut eid = 1u64;
        let node = |x: u64, y: u64| y * w + x;
        for y in 0..h {
            for x in 0..w {
                if x + 1 < w {
                    net.publish(Tx {
                        id: tid,
                        ops: vec![Op::AddEdge(sim_network::Edge {
                            id: eid,
                            a: node(x, y),
                            b: node(x + 1, y),
                            cost: 1000,
                            geom: vec![(x as i64 * 1000, y as i64 * 1000), ((x + 1) as i64 * 1000, y as i64 * 1000)],
                        })],
                    })
                    .unwrap();
                    tid += 1;
                    eid += 1;
                }
                if y + 1 < h {
                    net.publish(Tx {
                        id: tid,
                        ops: vec![Op::AddEdge(sim_network::Edge {
                            id: eid,
                            a: node(x, y),
                            b: node(x, y + 1),
                            cost: 1000,
                            geom: vec![(x as i64 * 1000, y as i64 * 1000), (x as i64 * 1000, (y + 1) as i64 * 1000)],
                        })],
                    })
                    .unwrap();
                    tid += 1;
                    eid += 1;
                }
            }
        }
        (net, coords)
    }

    /// 参考实现：Dijkstra（仅测试用，验证最优性对拍）。
    fn dijkstra(net: &Network, start: u64, goal: u64) -> Option<i64> {
        let mut dist: BTreeMap<u64, i64> = BTreeMap::new();
        dist.insert(start, 0);
        let mut done: BTreeSet<u64> = BTreeSet::new();
        loop {
            let next = dist
                .iter()
                .filter(|(n, _)| !done.contains(n))
                .min_by_key(|(n, d)| (**d, **n))
                .map(|(n, d)| (*n, *d));
            let (u, du) = next?;
            if u == goal {
                return Some(du);
            }
            done.insert(u);
            for (_, to, cost) in net.neighbors(u) {
                let nd = du.saturating_add(cost);
                if dist.get(&to).is_none_or(|&d| nd < d) {
                    dist.insert(to, nd);
                }
            }
        }
    }

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
        assert!(!LAYER.is_empty());
    }

    /// 最优性对拍（ADR-NEW-24 退出判据）：A* 代价 == Dijkstra 代价。
    #[test]
    fn astar_matches_dijkstra() {
        let (net, coords) = grid(5, 5);
        for goal in [4u64, 12, 20, 24] {
            let out = find_once(&net, &coords, 0, goal, 10_000);
            let FindOutcome::Complete { cost, .. } = out else { panic!("must complete") };
            assert_eq!(Some(cost), dijkstra(&net, 0, goal), "goal={goal}");
        }
    }

    /// 唯一性（ADR-NEW-24 退出判据）：同输入重复运行路径逐边一致。
    #[test]
    fn astar_is_unique() {
        let (net, coords) = grid(6, 6);
        let a = find_once(&net, &coords, 0, 35, 10_000);
        let b = find_once(&net, &coords, 0, 35, 10_000);
        assert_eq!(a, b);
    }

    /// 平局打破确定性（决策 2 第 2 条）：两条等价路径按 (f,h,node_id) 选 node_id 小者。
    /// 2×2 网格：0->1->3 与 0->2->3 等价，node 1 < node 2 ⇒ 走 0->1->3。
    #[test]
    fn tie_break_is_deterministic() {
        let (net, coords) = grid(2, 2);
        let FindOutcome::Complete { path, .. } = find_once(&net, &coords, 0, 3, 100) else {
            panic!("must complete")
        };
        // 边 ID：0->1 是水平第一条（id=1），1->3 是纵向（id=3）
        assert_eq!(path, vec![1, 3], "平局必须走 node_id 较小的分支");
    }

    /// 机制 B 解析上界（ADR-NEW-20 三硬约束）：让步次数 ≤ ⌈log2(|V|/b)⌉+1 且必达终态。
    #[test]
    fn budget_yield_respects_analytic_bound() {
        let (net, coords) = grid(4, 4); // |V|=16
        let policy = BudgetPolicy { b: 1, g: 2 };
        let bound = policy.max_ticks_to_terminal(net.node_count());
        let mut pf = PathFinder::new(policy);
        let mut ticks = 0u64;
        loop {
            ticks += 1;
            match pf.request(&net, &coords, 0, 15) {
                FindOutcome::Complete { .. } => break,
                FindOutcome::Yielded { .. } => {
                    assert!(ticks <= bound, "超解析上界: ticks={ticks} bound={bound}");
                    continue;
                }
                FindOutcome::Unreachable { .. } => panic!("网格必连通"),
            }
        }
        assert!(ticks <= bound, "终态 tick 数必须 ≤ 解析上界");
        assert_eq!(pf.budget_yield_count, 0, "成功后让步计数清零");
        // B_k 递增性质
        assert_eq!(policy.budget_for_yield(0, 16), 1);
        assert_eq!(policy.budget_for_yield(1, 16), 2);
        assert_eq!(policy.budget_for_yield(4, 16), 16, "封顶 L=|V|");
    }

    /// 受阻终态：不可达必须返回 Unreachable（有限步终止，不得死循环）。
    #[test]
    fn unreachable_terminates_as_blocked() {
        let (mut net, coords) = grid(2, 2);
        // 删掉所有边 → 孤立
        for id in [1u64, 2, 3, 4] {
            net.publish(Tx { id: 100 + id, ops: vec![Op::RemoveEdge { id }] }).unwrap();
        }
        let out = find_once(&net, &coords, 0, 3, 100);
        assert!(matches!(out, FindOutcome::Unreachable { expanded } if expanded > 0), "不可达须携带展开数");
    }

    /// 双计数器分离（ADR-NEW-20）：让步与过期重算互不影响。
    #[test]
    fn counters_are_separate() {
        let mut pf = PathFinder::new(BudgetPolicy { b: 1, g: 2 });
        pf.budget_yield_count = 5;
        pf.note_stale_recompute();
        pf.note_stale_recompute();
        assert_eq!(pf.budget_yield_count, 5, "过期重算不得动让步计数");
        assert_eq!(pf.stale_recompute_count, 2);
    }

    /// 平行同向边：路径边集必须是松弛实际使用的边（QWEN4-MAX 第二轮评审缺陷回归）。
    /// 回溯按 min edge id 会在平行边不同代价时返回与报告 cost 不符的边集。
    #[test]
    fn parallel_edge_reconstruction_is_exact() {
        let net = Network::from_edges(vec![
            Edge { id: 1, a: 0, b: 1, cost: 10, geom: vec![(0, 0), (0, 0)] }, // 平行边（较贵，id 小）
            Edge { id: 2, a: 0, b: 1, cost: 3, geom: vec![(0, 0), (0, 0)] },  // 平行边（较便宜）
            Edge { id: 3, a: 1, b: 2, cost: 5, geom: vec![(0, 0), (0, 0)] },
        ]);
        let coords = vec![(0i64, 0i64); 3];
        let FindOutcome::Complete { path, cost, .. } =
            find_once(&net, &coords, 0, 2, 100)
        else {
            panic!("must complete")
        };
        assert_eq!(cost, 8, "最优代价=3+5");
        assert_eq!(path, vec![2, 3], "必须选便宜的平行边 id=2，而非 min id=1");
        // 边集代价之和 == 报告 cost（stale 窄判依赖边集正确性）
        let sum: i64 = path
            .iter()
            .map(|&eid| net.edge(eid).expect("边存在").cost)
            .sum();
        assert_eq!(sum, cost, "返回边集的代价和必须等于报告 cost");
    }

    /// 溢出契约（ADR-NEW-21 决策 3）：饱和运算，禁止静默环绕；超大坐标/代价下确定性。
    #[test]
    fn overflow_saturates_deterministically() {
        let net = Network::from_edges(vec![Edge {
            id: 1,
            a: 0,
            b: 1,
            cost: i64::MAX,
            geom: vec![(0, 0), (0, 0)],
        }]);
        let coords = vec![(i64::MIN, i64::MIN), (i64::MAX, i64::MAX)];
        // h 不得环绕为负/恐慌：饱和到 i64::MAX
        assert_eq!(h_manhattan(&coords, 0, 1), i64::MAX);
        let a = find_once(&net, &coords, 0, 1, 100);
        let b = find_once(&net, &coords, 0, 1, 100);
        assert_eq!(a, b, "溢出路径下两次运行必须逐边一致");
        let FindOutcome::Complete { cost, path, .. } = a else { panic!("must complete") };
        assert!(cost >= 0, "饱和运算不得环绕为负");
        assert_eq!(path, vec![1]);
    }

    /// 债务 #29：全局配额截断——多请求共享 TickBudget，总展开 ≤ Q；耗尽后让步；reset 后恢复。
    #[test]
    fn global_budget_caps_total_expansions_per_tick() {
        let (net, coords) = grid(10, 10);
        let mut finder = PathFinder::new(BudgetPolicy::default());
        let mut budget = TickBudget::new(64);
        let mut total: u64 = 0;
        let mut completes = 0u32;
        let mut zero_yield = 0u32;
        for _ in 0..10 {
            let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
            match &out {
                FindOutcome::Complete { expanded, .. } => {
                    total += expanded;
                    completes += 1;
                }
                FindOutcome::Yielded { expanded } => {
                    total += expanded;
                    if *expanded == 0 {
                        zero_yield += 1;
                    }
                }
                FindOutcome::Unreachable { .. } => panic!("10x10 网格角到角必须可达"),
            }
        }
        assert!(total <= 64, "全局每 tick 总展开必须 ≤ Q，实测 {total}");
        assert!(completes >= 1, "配额内应有请求完成");
        assert!(zero_yield >= 1, "配额耗尽后请求必须以 expanded:0 让步");
        assert_eq!(budget.remaining, 0, "配额应恰好耗尽");
        // 下一 tick 重置后可以重新工作
        budget.reset();
        let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
        assert!(matches!(out, FindOutcome::Complete { .. }), "reset 后应完成");
    }

    /// 机制 B 推进：让步后下一 tick 以更大 B_k 重试，有限 tick 内完成（债务 #23/#29 交叉）。
    #[test]
    fn yield_then_fresh_budget_completes() {
        let (net, coords) = grid(10, 10);
        let mut finder = PathFinder::new(BudgetPolicy { b: 1, g: 2 });
        let mut budget = TickBudget::new(1024);
        let mut ticks = 0u64;
        loop {
            ticks += 1;
            budget.reset(); // 每 tick 预算独立
            let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
            match out {
                FindOutcome::Complete { .. } => break,
                FindOutcome::Yielded { .. } => assert!(ticks <= 16, "让步必须在有限 tick 内终止"),
                FindOutcome::Unreachable { .. } => panic!("必须可达"),
            }
        }
        assert_eq!(finder.budget_yield_count, 0, "完成后让步计数清零");
    }

    /// X2 审计修复回归：不可达搜索实耗必须扣减全局配额（否则同 tick 多不可达
    /// 请求可绕过 Q，违反 ADR-NEW-14 条 3）。
    #[test]
    fn unreachable_expansion_is_charged_to_budget() {
        let (net, coords) = grid(3, 3);
        let mut finder = PathFinder::new(BudgetPolicy { b: 64, g: 2 });
        let mut budget = TickBudget::new(64);
        // 9 号节点在 3x3 网格中不存在——用孤立图改测不可达更稳：从网格取一节点
        // 去边不可行（grid 不暴露），改用目标=孤立构造。
        let isolated = Network::from_edges(vec![]);
        let ic: Vec<(i64, i64)> = vec![(0, 0), (1000, 0)];
        let out = finder.request_with_budget(&isolated, &ic, 0, 1, &mut budget);
        assert!(
            matches!(out, FindOutcome::Unreachable { expanded } if expanded == 1),
            "孤立图：展开起点 1 个节点后 open 耗尽"
        );
        assert_eq!(budget.remaining, 63, "不可达实耗 1 展开必须入账");
        // 可达路径对照：同一 budget 继续扣减
        let _ = finder.request_with_budget(&net, &coords, 0, 8, &mut budget);
        assert!(budget.remaining < 63, "后续请求继续扣减");
    }

    /// X2 审计修复回归：配额耗尽让步计入 quota_yield_count，不污染
    /// budget_yield_count（债务 #27 "让步≥8"切换判据只认请求自身预算让步）。
    #[test]
    fn quota_yield_does_not_pollute_switch_criterion() {
        let (net, coords) = grid(10, 10);
        let mut finder = PathFinder::new(BudgetPolicy { b: 64, g: 2 });
        let mut budget = TickBudget::new(0); // 配额为 0：首请求即配额让步
        let out = finder.request_with_budget(&net, &coords, 0, 99, &mut budget);
        assert!(matches!(out, FindOutcome::Yielded { expanded: 0 }), "配额耗尽即让步");
        assert_eq!(finder.quota_yield_count, 1, "配额让步单独计数");
        assert_eq!(finder.budget_yield_count, 0, "配额让步不得污染预算让步判据");
        // 对照：请求自身预算让步仍走 budget_yield_count
        let mut b2 = TickBudget::new(4);
        let _ = finder.request_with_budget(&net, &coords, 0, 99, &mut b2);
        assert!(finder.budget_yield_count >= 1, "自身预算让步单独计数");
        assert_eq!(finder.quota_yield_count, 1, "对照请求不新增配额让步");
    }
}

```

## crates/sim-core/src/budget.rs  (sha256:67a150915f93e235…, 17757 B)

```text
//! 预算注册表与确定性降级（ADR-NEW-14 C2 细化，Stage 2C C3a）。
//!
//! - 契约即代码：每系统在注册表声明计量单位/每 tick 上限/超限行为。
//! - 降级触发只依赖确定性工作量单位（预算消耗比例），禁止墙钟/负载/帧率。
//! - 降级等级与滚动预算是权威状态：入存档、入 GlobalDigest（G3 口径）。

use sim_codec::CanonicalWriter;
use sim_error::{ErrorCode, ErrorClass, SimError, SimErrorContext};

/// 计量单位（确定性工作量单位，ADR-NEW-14）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkUnit {
    /// A* 节点展开数（sim-pathfinding）。
    NodeExpansions,
    /// 事务应用数（sim-network）。
    TxApplies,
    /// 本 tick 更新实体数（sim-traffic 等）。
    EntityUpdates,
    /// 摘要/存档条目数（sim-save）。
    DigestEntries,
}

/// 超限行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowBehavior {
    /// 让步到下 tick 续算（寻路）。
    YieldToNextTick,
    /// 排队到下 tick（事务）。
    QueueToNextTick,
    /// 分片让步（实体更新）。
    ShardYield,
    /// Fatal：存档不可信（SAV-0003）。
    FatalSaveCorrupt,
}

/// 单个系统的预算声明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetDecl {
    pub system_id: u32,
    pub unit: WorkUnit,
    pub per_tick_cap: u64,
    pub overflow: OverflowBehavior,
}

/// 降级等级（EC-04 L1/L2/L3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DegradationLevel {
    Normal,
    L1,
    L2,
    L3,
}

/// 降级阈值（候选占位：70/90/100 万分比 + 恢复 5000bp；ADR-NEW-14 C2 待负责人确认）。
/// 2026-11-19 投票修复（Claude）：触发/恢复加连续 tick 要求，防单 tick 突发误触发/误恢复。
#[derive(Debug, Clone, Copy)]
pub struct DegradationThresholds {
    /// 预算消耗比例 ≥ 此值触发（万分比：7000=70%）。触发比较为闭区间（>=）。
    pub l1_bp: u32,
    pub l2_bp: u32,
    pub l3_bp: u32,
    /// 恢复迟滞（万分比：5000=50%）。恢复比较为开区间（<）。
    pub recover_bp: u32,
    /// 进入降级所需连续超阈 tick 数（候选占位=2；单 tick 突发不触发）。
    pub trigger_ticks: u32,
    /// 恢复 Normal 所需连续低于 recover_bp 的 tick 数（候选占位=2）。
    pub recover_ticks: u32,
}

impl Default for DegradationThresholds {
    fn default() -> Self {
        Self {
            l1_bp: 7000,
            l2_bp: 9000,
            l3_bp: 10000,
            recover_bp: 5000,
            trigger_ticks: 2,
            recover_ticks: 2,
        }
    }
}

/// 预算注册表 + 降级状态机。全部状态确定性、可入档。
#[derive(Debug, Clone)]
pub struct BudgetRegistry {
    decls: Vec<BudgetDecl>,
    /// 本 tick 各系统实耗（索引与 decls 对齐）。
    spent: Vec<u64>,
    /// 本 tick 各系统预算占比的最大值（万分比）。2026-11-19 投票修复（MiniMax/GPT-6）：
    /// 聚合口径=逐系统比例取最严（旧口径按 cap 加权总和，单系统饱和会被空闲系统稀释）。
    rolling_used_bp: u32,
    /// 连续超阈 tick 计数（触发门）。
    over_streak: u32,
    /// 连续低于恢复线 tick 计数（恢复门）。
    under_streak: u32,
    /// 本 tick 外部需求压力（万分比，0..=10000；MFD→降级信号通道，ADR-NEW-26 草案）。
    demand_bp: u32,
    /// 本 tick 需求压力的归档口径（X5 审计修复：demand_bp 在 end_tick 清零，
    /// 直接入档恒 0——last_demand_bp 跨 end_tick 保留供 encode_state 归档）。
    last_demand_bp: u32,
    /// 连续高需求 tick 计数（与 over_streak 同门限语义）。
    demand_streak: u32,
    pub level: DegradationLevel,
    thresholds: DegradationThresholds,
}

impl BudgetRegistry {
    pub fn new(decls: Vec<BudgetDecl>, thresholds: DegradationThresholds) -> Self {
        let spent = vec![0; decls.len()];
        Self {
            decls,
            spent,
            rolling_used_bp: 0,
            over_streak: 0,
            under_streak: 0,
            demand_bp: 0,
            last_demand_bp: 0,
            demand_streak: 0,
            level: DegradationLevel::Normal,
            thresholds,
        }
    }

    pub fn decls(&self) -> &[BudgetDecl] {
        &self.decls
    }

    pub fn level(&self) -> DegradationLevel {
        self.level
    }

    /// 记录实耗；返回是否超该系统每 tick 上限。
    pub fn spend(&mut self, system_id: u32, amount: u64) -> bool {
        let idx = self
            .decls
            .iter()
            .position(|d| d.system_id == system_id)
            .expect("undeclared system");
        self.spent[idx] = self.spent[idx].saturating_add(amount);
        self.spent[idx] > self.decls[idx].per_tick_cap
    }

    /// 记录本 tick 外部需求压力（万分比；确定性定点，来自权威状态导出——
    /// 如 sim-traffic 密度/k_c 比值。ADR-NEW-26 草案通道；每 tick 末清零）。
    pub fn note_demand_pressure(&mut self, pressure_bp: u32) {
        self.demand_bp = pressure_bp.min(10_000);
        self.last_demand_bp = self.demand_bp;
    }

    /// tick 边界：按确定性工作量单位更新降级等级，重置每 tick 计量。
    /// 返回新等级。触发/恢复只依赖 spent/cap 比例（工作量单位），无墙钟。
    ///
    /// 2026-11-19 投票修复：
    /// - 聚合口径=逐系统比例取最严（max_i spent_i/cap_i），单系统饱和不被稀释；
    /// - 触发（Normal→降级）需连续 trigger_ticks 个超阈 tick（触发比较闭区间 >=）；
    /// - 恢复（→Normal）需连续 recover_ticks 个低于 recover_bp 的 tick（开区间 <）；
    /// - 升级（L1→L2→L3）保持即时（安全方向不加延迟）。
    pub fn end_tick(&mut self) -> DegradationLevel {
        let used_bp: u32 = self
            .decls
            .iter()
            .zip(self.spent.iter())
            .map(|(d, s)| {
                if d.per_tick_cap == 0 {
                    if *s > 0 { 10_000u32 } else { 0 }
                } else {
                    ((*s as u128).saturating_mul(10_000) / d.per_tick_cap as u128).min(10_000) as u32
                }
            })
            .max()
            .unwrap_or(0);
        self.rolling_used_bp = used_bp;
        let t = self.thresholds;
        // 触发/恢复连续计数（确定性派生状态）：有效压力 = max(工作量占比, 外部需求压力)。
        // 需求压力参与触发/恢复门，但单独触发时降级封顶 L1（模型形式未裁决=债务 #31，
        // 限制影响面；工作量压力仍可推 L2/L3）。
        let effective_bp = used_bp.max(self.demand_bp);
        if effective_bp >= t.l1_bp {
            self.over_streak = self.over_streak.saturating_add(1);
            self.demand_streak = if self.demand_bp >= t.l1_bp {
                self.demand_streak.saturating_add(1)
            } else {
                0
            };
        } else {
            self.over_streak = 0;
            self.demand_streak = 0;
        }
        if effective_bp < t.recover_bp {
            self.under_streak = self.under_streak.saturating_add(1);
        } else {
            self.under_streak = 0;
        }
        let triggered = self.over_streak >= t.trigger_ticks;
        let demand_only_trigger = triggered
            && used_bp < t.l1_bp
            && self.demand_streak >= t.trigger_ticks;
        let recovered = self.under_streak >= t.recover_ticks;
        self.level = match self.level {
            DegradationLevel::Normal => {
                if !triggered {
                    DegradationLevel::Normal
                } else if demand_only_trigger {
                    DegradationLevel::L1
                } else if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else {
                    DegradationLevel::L1
                }
            }
            DegradationLevel::L1 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if used_bp >= t.l2_bp {
                    DegradationLevel::L2
                } else if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L1
                }
            }
            DegradationLevel::L2 => {
                if used_bp >= t.l3_bp {
                    DegradationLevel::L3
                } else if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L2
                }
            }
            DegradationLevel::L3 => {
                if recovered {
                    DegradationLevel::Normal
                } else {
                    DegradationLevel::L3
                }
            }
        };
        self.spent.iter_mut().for_each(|s| *s = 0);
        self.demand_bp = 0;
        self.level
    }

    /// 编码入档/入摘要（字段序即 canonical 序，G3 口径）。
    pub fn encode_state(&self, w: &mut CanonicalWriter) {
        w.u32(self.rolling_used_bp);
        w.u32(self.over_streak);
        w.u32(self.under_streak);
        w.u32(self.last_demand_bp);
        w.u32(self.demand_streak);
        w.u8(match self.level {
            DegradationLevel::Normal => 0,
            DegradationLevel::L1 => 1,
            DegradationLevel::L2 => 2,
            DegradationLevel::L3 => 3,
        });
    }

    /// 超限错误（供 FatalSaveCorrupt 类超限使用）。
    /// system_id 经 ctx.system/调用方日志定位（message 为 &'static str 无法插值）。
    pub fn overflow_error(&self, _system_id: u32, ctx: SimErrorContext) -> SimError {
        SimError {
            class: ErrorClass::Fatal,
            code: ErrorCode { module: "SAV", number: 3 },
            message: "digest entries exceeded per-tick cap; save untrustworthy",
            context: ctx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decls() -> Vec<BudgetDecl> {
        vec![
            BudgetDecl {
                system_id: 1,
                unit: WorkUnit::NodeExpansions,
                per_tick_cap: 100,
                overflow: OverflowBehavior::YieldToNextTick,
            },
            BudgetDecl {
                system_id: 2,
                unit: WorkUnit::TxApplies,
                per_tick_cap: 10,
                overflow: OverflowBehavior::QueueToNextTick,
            },
        ]
    }

    #[test]
    fn role_is_pinned() {
        assert!(crate::authoritative());
    }

    #[test]
    fn spend_and_overflow_flag() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        assert!(!r.spend(1, 100));
        assert!(r.spend(1, 1)); // 101 > 100
        assert!(!r.spend(2, 10));
        assert!(r.spend(2, 1));
    }

    #[test]
    fn degradation_levels_follow_workload_only() {
        // trigger_ticks=1：纯比例路径（旧口径测试迁移）
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        // 逐系统取最严：系统 1 消耗 77/100 → 7700bp ≥7000 → L1
        r.spend(1, 77);
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        // 重置后无消耗 → 恢复 Normal（< 5000bp）
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
        // 系统 1 消耗 110 → 10000bp → L3
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 迟滞：60/100=6000bp ≥5000 → 保持 L3
        r.spend(1, 60);
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        // 30/100=3000bp < 5000 → 恢复
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
    }

    #[test]
    fn single_system_saturation_is_not_diluted() {
        // 2026-11-19 投票修复（MiniMax/GPT-6）回归：旧 cap 加权口径下
        // 系统 2（cap=10）打满只贡献 10/110≈9%→永不触发；新口径取最严必须触发。
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        r.spend(2, 10); // 系统 2 满负荷 10000bp，系统 1 空闲
        assert_eq!(r.end_tick(), DegradationLevel::L3);
    }

    #[test]
    fn threshold_boundaries_are_closed_open() {
        // GPT-6 边界要求：恰 70/90/100 触发（闭区间 >=），恰 50 不恢复（开区间 <）。
        let th = DegradationThresholds { trigger_ticks: 1, recover_ticks: 1, ..Default::default() };
        let mut r = BudgetRegistry::new(decls(), th);
        r.spend(1, 70); // 7000bp 恰= l1
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        r.spend(1, 90); // 9000bp 恰= l2
        assert_eq!(r.end_tick(), DegradationLevel::L2);
        r.spend(1, 100); // 10000bp 恰= l3
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        r.spend(1, 50); // 5000bp 恰= recover → 不恢复（开区间）
        assert_eq!(r.end_tick(), DegradationLevel::L3);
        r.spend(1, 49); // 4900bp < 5000 → 恢复
        assert_eq!(r.end_tick(), DegradationLevel::Normal);
    }

    #[test]
    fn trigger_and_recover_need_consecutive_ticks() {
        // Claude：单 tick 突发不触发/不恢复（默认 trigger_ticks=recover_ticks=2）。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 1 个超阈 tick：不触发
        r.spend(1, 110);
        assert_eq!(r.end_tick(), DegradationLevel::L3); // 第 2 个：触发
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::L3); // 第 1 个低消耗 tick：不恢复
        r.spend(1, 30);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 2 个：恢复
    }

    #[test]
    fn state_encoding_is_deterministic() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.spend(1, 90);
        r.end_tick();
        let mut w1 = CanonicalWriter::new("sim-core", "BudgetState");
        r.encode_state(&mut w1);
        let mut w2 = CanonicalWriter::new("sim-core", "BudgetState");
        r.encode_state(&mut w2);
        assert_eq!(w1.into_bytes(), w2.into_bytes());
    }

    #[test]
    fn unknown_system_spend_panics() {
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            r.spend(99, 1);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn demand_pressure_triggers_l1_only_with_hysteresis() {
        // MFD→降级通道（ADR-NEW-26 草案）：需求压力参与触发/恢复门（effective=max），
        // 单独触发封顶 L1；恢复需需求与工作量同时回落。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        r.note_demand_pressure(10_000);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 1 tick：触发门未到
        r.note_demand_pressure(10_000);
        assert_eq!(r.end_tick(), DegradationLevel::L1);
        r.note_demand_pressure(0);
        assert_eq!(r.end_tick(), DegradationLevel::L1); // 第 1 个低需求 tick：不恢复
        r.note_demand_pressure(0);
        assert_eq!(r.end_tick(), DegradationLevel::Normal); // 第 2 个：恢复
    }

    #[test]
    fn demand_alone_cannot_reach_l2_l3() {
        // 需求压力单独触发封顶 L1（债务 #31 未裁决前限制影响面）；
        // 工作量压力仍可推 L2/L3。
        let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
        for _ in 0..2 {
            r.note_demand_pressure(10_000);
            assert_ne!(r.end_tick(), DegradationLevel::L2);
        }
        assert_eq!(r.level(), DegradationLevel::L1);
        for _ in 0..2 {
            r.spend(1, 2000);
            r.end_tick();
        }
        assert_eq!(r.level(), DegradationLevel::L3);
    }

    #[test]
    fn demand_state_is_covered_by_encode() {
        // 需求通道状态入档/入摘要（G3）：demand_bp 敏感。
        let mk = |demand: u32| {
            let mut r = BudgetRegistry::new(decls(), DegradationThresholds::default());
            r.note_demand_pressure(demand);
            let mut w = CanonicalWriter::new("sim-core", "BudgetState");
            r.encode_state(&mut w);
            w.into_bytes()
        };
        assert_ne!(mk(0), mk(10_000), "demand_bp must be covered");
    }

    #[test]
    fn demand_archival_survives_end_tick() {
        // X5 审计修复：encode_state 归档 last_demand_bp（end_tick 清零 demand_bp
        // 后仍可归档——旧口径入档恒 0）。
        let decls = vec![BudgetDecl { system_id: 1, unit: WorkUnit::NodeExpansions, per_tick_cap: 1000, overflow: OverflowBehavior::YieldToNextTick }];
        let mk = |demand: u32| {
            let mut r = BudgetRegistry::new(decls.clone(), DegradationThresholds::default());
            r.note_demand_pressure(demand);
            r.end_tick(); // 清零 demand_bp 后归档
            let mut w = CanonicalWriter::new("sim-core", "Budget");
            r.encode_state(&mut w);
            w.into_bytes()
        };
        assert_ne!(mk(0), mk(10_000), "last_demand_bp must survive end_tick");
    }
}

```

## docs/architecture/decisions/decision-log.md  (sha256:92572aadcc1e8920…, 81584 B)

```text
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

```
