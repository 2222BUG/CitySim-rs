# CitySim-rs 决策日志（decision-log）

本文件记录 ADR 的状态与变更历史。编号为整合后的暂定索引，正式入仓时按仓库 ADR 序列重新编号并保留映射。

## V1.9.3 新增决策（源自五项修订提案，2026-11-18 并入主文档）

| ADR | 主题 | 当前状态 | 裁决 | 待决事项 |
| --- | --- | --- | --- | --- |
| ADR-NEW-14 | 预算与降级必须确定性 | Proposed | 待评审 | 各系统工作量计量单位与每 tick 上限阈值；R2/R3 分层后门禁实测耗时记录于 docs/benchmarks/；预算滚动状态生命周期（见债务登记 #20） |
| ADR-NEW-15 | 确定性边界定义 | Proposed | 待评审 | clippy disallowed-types/methods 分层配置可行性验证；跨平台一致性（R4）；Rapier ADR 豁免机制（若承担权威物理） |
| ADR-NEW-16 | 存档最小集合扩充 | Proposed | 待评审 | 在途任务恢复策略已定案为"保存并恢复"（2026-11-18 评审修订）；永久兼容边界；存档序列化格式（见债务登记 #11） |
| ADR-NEW-17 | 路网事务的过期结果策略 | Proposed | 待评审 | 相交判断粒度、重算上限数值、重算超限兜底行为比选（局部可行路径/沿用旧路径有效段/标记受阻）、行走中道路被删除的行为（停止/回退/就近重算）；影响区域表示法（见债务登记 #9） |
| ADR-NEW-18 | 仿真参数基线与数值契约补充 | Proposed | 待评审 | 参数基线七项已于 2026-11-18 由项目负责人定案（Cities: Skylines 2 参考值，见 ADR-NEW-18 定案记录）；舍入规则 Rust 实测已有初值（spikes/simscalar-spike 黄金测试 7/7），正式定案待裁决；分析证据已入仓 docs/benchmarks/；待办=用定案参数重跑 Spike A（Step 2） |

裁决口径（见主文档第 7 章）：采纳 / 有条件采纳 / 拒绝 / 延后，每项裁决须记录理由和验证条件。

## 2026-11-18 评审后新增 ADR 草案（源自设计审计）

来源：开工前设计审计（25 条发现，其中阻塞级 3 条）。审计结论：写代码前先补三件事——GlobalDigest ADR、请求 ID/在途任务口径定案、未登记债务入账。前两件立项如下，第三件见下一节。

| ADR | 主题 | 当前状态 | 裁决 | 待决事项 |
| --- | --- | --- | --- | --- |
| ADR-NEW-19 | GlobalDigest 摘要算法与规范化序列化 | Proposed | 待评审 | 哈希算法定案（BLAKE3-256 推荐 / xxh3-64 备选）；canonical 编码器选型（bincode 固定配置或自定义）；纳入范围模块清单定案；digest_version bump 规则 |
| ADR-NEW-20 | 请求/事务 ID 分配与在途寻路任务口径 | Proposed | 待评审 | 单 tick 原子搜索的性能上限（Spike C 验证；若超限则切换备选①存档含部分搜索状态并修订 ADR）；请求/事务 ID 高水位随 EC-03 存档最小集入档 |
| ADR-NEW-21 | SimScalar 数值格式定案（Spike A 数值设计决策） | Proposed | 待评审 | 裁决后 2.4 由"候选"改 Accepted 并同步编号映射；跨平台黄金值复验（R4）；Rust 基线 1.89 vs 实测 1.100.0 差异收敛 |
| ADR-NEW-22 | 系统级 RNG 协议（Spike B 协议设计决策） | Proposed | 待评审 | 统计质量正式测试（TestU01/PractRand 级）是否补充；namespace 注册表随债务 #15 定案；跨平台黄金值复验（R4） |

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
| 9 | 影响区域表示法（矩形 / 边集 / 瓦片）与请求/结果字段 | EC-01 | 重要 | Spike C |
| 10 | 回放输入 / 模拟配置 schema | 2.7、2.9 条 8、EC-11 | 重要 | Stage 1 前 |
| 11 | 存档序列化格式（与摘要 canonical 编码共用编码器） | EC-03 | 重要 | Stage 1 前，随 ADR-NEW-19 编码器定案 |
| 12 | Schema / 确定性协议 / RNG 协议版本号规则与 bump 策略 | EC-03 | 一般 | Stage 1 前 |
| 13 | 2.8"需要明确"五项（快照机制/结果排序/冲突处理/失败任务/Tick 边界） | 2.8 | 重要 | Stage 1 前 |
| 14 | LOD 三级命名/切换条件/更新频率；LOD 无 Stage 归属 | 2.7、ADR-NEW-14 | 重要 | 排入第五章阶段计划，Stage 2C 前 |
| 15 | crate 权威/非权威分类清单 | 2.2、EC-12 | 重要 | 与 clippy 分层实测同期 |
| 16 | 预算声明载体与 L1/L2/L3 按系统映射 | 2.7、EC-04 | 一般 | Stage 2C 前 |
| 17 | A* 确定性细则（平局打破/邻居展开序/启发式舍入）；Stage 2A 退出判据 | 2.4、第 5 章 | 重要 | Stage 2A 前 |
| 18 | EC-01 取消语义 / 冲突合并 / 验证判据 | EC-01 | 一般 | Spike C |
| 19 | Stage 1 任务清单与原始路线图缺失（须重建入仓） | 第 5/8 章 | 重要 | Stage 1 开工前 |
| 20 | 预算滚动状态生命周期（若每 tick 重置则 tick 边界存档无意义） | EC-03、2.7 | 一般 | ADR-NEW-14 修订定案 |

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

注：本表裁决针对评审建议本身。五项修订提案（ADR-NEW-14 至 ADR-NEW-18）的整体裁决仍为"待评审"，需按主文档第 7 章流程逐条完成正式裁决。

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

## Stage 0 工程步骤执行日志

按主文档第 5 章 Stage 0 重点、第 8 章优先级顺序逐步执行；每步一个 git 提交（提交主题见"提交"列）。

| 步骤 | 内容 | 状态 | 提交 |
| --- | --- | --- | --- |
| Step 1 | 补齐 2.10 参数基线（第零优先，负责人定案） | 已完成（2026-11-18） | docs(step1): 2.10 参数基线定案（CS2 参考） |
| Step 2 | Spike A：SimScalar（定案参数重跑 + Rust 范围/漂移验证 + 性能数据 + 数值设计决策 ADR） | 已完成（2026-11-18） | docs(step2): Spike A 收尾——定案核对与 SimScalar 定案建议 |
| Step 3 | Spike B：系统级 RNG（原型 + 确定性测试 + 算法比较 + ADR） | 已完成（2026-11-18） | docs(step3): Spike B——系统级 RNG 原型、9 测试、ADR-NEW-22 |
| Step 4 | Spike C：路网事务原子发布（原型 + 并发测试 + 过期结果统计 + 行走删路比选 + ADR） | 待开始 | - |
| Step 5 | 存档兼容验证（EC-03 扩充字段 + 读档等价测试） | 待开始 | - |
| Step 6 | 测试基础设施（EC-07 sim-test-support + 确定性注入） | 待开始 | - |
| Step 7 | 基础 CI（R0/R1 门禁骨架 + clippy 分层实测） | 待开始 | - |
| Step 8 | 依赖与工具链验证（rust-toolchain 锁定 + 五件套编译验证） | 待开始 | - |
| Step 9 | 核心工程 workspace 稳定编译 + Stage 0 退出条件核对 | 待开始 | - |

## 变更历史

| 日期 | 事件 |
| --- | --- |
| 2026-11-18 | 五项修订提案并入技术方案 V1.9.3；建立 ADR-NEW-14 至 ADR-NEW-18 独立文件；全部标记 Proposed |
| 2026-11-18 | 首轮评审 13 项建议裁决并落实（采纳 11 项、有条件采纳 2 项）；ADR-NEW-14/15/16/17/18 增加修订记录；主文档 V1.9.3 同步修订（草案并入，未裁决条文仍标 Proposed） |
| 2026-11-18 | 设计审计（25 条发现）与 Spike A 四件套评审完成：Spike A 证据与原型入仓（docs/benchmarks/、spikes/simscalar-spike/，cargo test 7/7 @ rustc 1.100.0）；tick 频率漂移决策数据入仓；新增 ADR-NEW-19/20 草案（状态 Proposed）；未登记决策债务入账 20 项；主文档二轮随手修（含新增 6.5 节）；裁决口径不变，全部待正式评审 |
| 2026-11-18 | Step 1（Stage 0 第零优先）：2.10 参数基线七项由项目负责人定案（参照 Cities: Skylines 2 同类数值 + Spike A / tick-rate 分析；tick 30 Hz、地图 16 km、分辨率 1 mm、速度 50 m/s、加速度 10 m/s²、经济 i64 10^12、参考场景 100k Agent、最大持续 10^9 ticks）；主文档 2.10（MD+docx）、ADR-NEW-18 定案记录同步；建立 Stage 0 工程步骤执行日志 |
| 2026-11-18 | Step 2（Spike A 收尾）：定案参数正式核对（simscalar-formal-check.py，9 项硬约束全 PASS，推荐 Q32.32）；spike crate 增 5 测试共 12 全过（dt 30 Hz 黄金值、漂移黄金、位宽、16 km/1 mm 容纳、性能冒烟 2.5 ns/op）；新增 ADR-NEW-21（SimScalar 格式定案，状态 Proposed）；2.4 加定案建议注；证据入仓 docs/benchmarks/（formal-check、simscalar-perf） |
| 2026-11-18 | Step 3（Spike B）：系统级 RNG 原型入仓 spikes/rng-spike（零依赖，9 测试全过：系统/Agent 隔离、调度无关、黄金复现、协议版本钉死、存档差异 0 vs 16 B）；算法比较（SplitMix64 0.59 ns 推荐 / xoshiro256** 0.73 / PCG64 1.14）；新增 ADR-NEW-22（RNG 协议，状态 Proposed）；EC-02 加 Spike 完成注；证据入仓 docs/benchmarks/（Spike-B-RNG-REPORT、rng-algo-comparison） |

## 关联

- 现行主文档：docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md（正式版，Markdown）；评审流转副本为工作区根目录 CitySim-rs技术方案V1.9.3.docx。
- 历史版本与回退依据：docs/architecture/history/（CitySim-rs V1.9.2 技术方案、CitySim-rs V1.9.2 修订提案（五项）原件、CitySim-rs-ADR文档包（修订前）.zip）。
- 历史决策（ADR-NEW-01 至 ADR-NEW-13 及原有 ADR）索引见主文档第 6 章。ADR-NEW-01 至 ADR-NEW-13 独立建档排期：V1.9.3 正式版发布前补齐，与 ADR-NEW-14 至 20 保持同等建档粒度（评审建议 13）。
- 证据归档：docs/benchmarks/——analysis.py（复核重建版）、analysis-output.txt、tick-rate-drift.py、tick-rate-drift-output.txt、Spike-A-SimScalar-REPORT.md（修订版）。
- Spike A 原型：spikes/simscalar-spike/（SimScalar Q32.32 定点原型 + 7 项黄金测试；rustc 1.100.0 实测 7/7，文档基线 1.89 待收敛——见二轮 P7）。
