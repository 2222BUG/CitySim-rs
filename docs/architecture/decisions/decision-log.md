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
| 8 | 路网数据字典；CSR 定案表述矛盾（Accepted vs 待性能测试） | 2.1、2.5 | 重要 | Stage 2A 前 |
| 9 | 影响区域表示法（矩形 / 边集 / 瓦片）与请求/结果字段 | EC-01 | 重要 | 已立项 ADR-NEW-23（Spike C 已完成 2026-11-18：轴对齐包围盒 1 mm 网格 + 并集聚合粒度），待评审裁决 |
| 10 | 回放输入 / 模拟配置 schema | 2.7、2.9 条 8、EC-11 | 重要 | Stage 1 前 |
| 11 | 存档序列化格式（与摘要 canonical 编码共用编码器） | EC-03 | 重要 | Stage 1 前，随 ADR-NEW-19 编码器定案；canonical 编码方向已由 save-restore-spike 验证（小端 + u64 长度前缀 + 尾字节拒绝，Step 5） |
| 12 | Schema / 确定性协议 / RNG 协议版本号规则与 bump 策略 | EC-03 | 一般 | Stage 1 前 |
| 13 | 2.8"需要明确"五项（快照机制/结果排序/冲突处理/失败任务/Tick 边界） | 2.8 | 重要 | Stage 1 前 |
| 14 | LOD 三级命名/切换条件/更新频率；LOD 无 Stage 归属 | 2.7、ADR-NEW-14 | 重要 | 排入第五章阶段计划，Stage 2C 前 |
| 15 | crate 权威/非权威分类清单 | 2.2、EC-12 | 重要 | clippy 分层实测已完成（2026-11-18）；清单落点=各权威 crate 包根 clippy.toml + CI ec12-lint 矩阵成员列表（.github/workflows/ci.yml），待正式权威 crate 建库时随 2.2 目录结构定案 |
| 16 | 预算声明载体与 L1/L2/L3 按系统映射 | 2.7、EC-04 | 一般 | Stage 2C 前 |
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
- [ ] ADR-NEW-20：G5 压测后 A/B/C 机制锁定（默认候选 B+三硬约束）
- [ ] ADR-NEW-20：每 tick 节点上限、最大等待 tick 上限与受阻衔接定值（随 NEW-14 阈值）
- [ ] ADR-NEW-20：请求/事务 ID 单一分配点定案（债务 #24）（Stage 1 进展：T3/T7 已实装集中单一分配点（CommandIdAllocator / Allocators）；tick 内固定相位取号规则随调度器（Stage 2B）钉死）
- [ ] ADR-NEW-21：R4 跨平台黄金值复验（dt_raw、drift_golden）（跨平台待 R4）
- [ ] ADR-NEW-21：正式基准替代冒烟值（2.5 ns/op 不得作门禁）
- [x] ADR-NEW-22：namespace 注册表+注册期碰撞校验（债务 #25）——已核销（2026-11-19，T5：sim-rng NamespaceRegistry 注册期 64 位碰撞拒绝 + 构造碰撞测试）
- [ ] ADR-NEW-22：TestU01/PractRand 统计测试或书面豁免（SplitMix64 统计充分性）
- [ ] ADR-NEW-22：协议版本不符处理随 EC-03 兼容矩阵（债务 #12）回填
- [ ] ADR-NEW-23：6 场景矩阵验收指标实测入 docs/benchmarks/（误判率/重算率/阻塞率/等待 tick）
- [ ] ADR-NEW-23：PartialPath/UseValidPrefix 启用前清债务 #18

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

## 关联

- 现行主文档：docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md（正式版，Markdown）；评审流转副本为工作区根目录 CitySim-rs技术方案V1.9.3.docx。
- 历史版本与回退依据：docs/architecture/history/（CitySim-rs V1.9.2 技术方案、CitySim-rs V1.9.2 修订提案（五项）原件、CitySim-rs-ADR文档包（修订前）.zip）。
- 历史决策（ADR-NEW-01 至 ADR-NEW-13 及原有 ADR）索引见主文档第 6 章。ADR-NEW-01 至 ADR-NEW-13 独立建档排期：V1.9.3 正式版发布前补齐，与 ADR-NEW-14 至 20 保持同等建档粒度（评审建议 13）。
- 证据归档：docs/benchmarks/——analysis.py（复核重建版）、analysis-output.txt、tick-rate-drift.py、tick-rate-drift-output.txt、Spike-A-SimScalar-REPORT.md（修订版）。
- Spike A 原型：spikes/simscalar-spike/（SimScalar Q32.32 定点原型 + 7 项黄金测试；rustc 1.100.0 实测 7/7，文档基线 1.89 待收敛——见二轮 P7）。
