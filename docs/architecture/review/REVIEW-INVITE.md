# CitySim-rs V1.9.3 公开评审邀请包

- 日期：2026-11-18
- 用途：把仓库评审（ADR-NEW-14~23 裁决）分享给外部评审人
- 评审依据：主文档第 7 章（docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md）
- 裁决口径：**采纳 / 有条件采纳 / 拒绝 / 延后**，每项记录**理由和验证条件**（记入 decision-log）

## 参与方式（给评审人的话）

1. 读 [README.md](../../README.md) → 主文档第 7 章 → 对应 ADR 文件（docs/architecture/adr/）。
2. 在对应 issue 下回复裁决意见，按下述模板填写；也欢迎直接提 Pull Request。
3. 每份 ADR 都标 **Proposed**——你的意见会经工程裁决后记入 decision-log，不会不加消化直接采纳。

## 12 个评审问题（主文档第 7 章原文）

1. SimScalar 的数值格式、位宽和误差是否满足长期模拟要求？（前提：2.10 参数基线定案）
2. 系统级 RNG 如何保证隔离、跨平台一致性及版本兼容？
3. 路网事务如何保证原子发布，并处理并发修改冲突？
4. 过期寻路结果的"影响区域相交判断"策略在频繁修路场景下的负载和活锁风险如何量化？
5. CSR 分块和 A* 调度是否适合目标规模？
6. 确定性摘要应覆盖哪些权威状态？
7. 2.9 确定性契约中哪些条款可以转为自动检查（clippy disallowed-types 等），哪些只能靠评审？
8. 多世界抽象是否会削弱 Bevy ECS 的调度能力？
9. 对象池是否确实能在目标负载下带来性能收益？
10. 10,000 Agent × 1,000 Tick 的测试如何控制运行时间和资源占用？叠加线程数无关性与读档等价测试后，R2 门禁耗时是否仍可接受？
11. Demo 应展示哪些真实完成的能力？
12. 各项技术依赖是否有可复现的编译和兼容性证据？

---

## Issue 草稿（复制即用）

> 建议命名：#1 对应 ADR-NEW-14，以此类推 #10 对应 ADR-NEW-23；另建一条 Meta（见下）。
> 自动化：`create-issues.ps1`（需 gh CLI 登录后运行，见文末）。

### Meta

**标题**：`[评审邀请] CitySim-rs V1.9.3 公开评审——ADR-NEW-14~23 裁决`

**正文**：

```
CitySim-rs 技术方案 V1.9.3 处于公开评审阶段（未冻结架构）。本仓库已完成 Stage 0 工程验证（9 步、51 测试、8 份实测报告），现邀请对 10 份 Proposed 状态的 ADR 做正式裁决。

- 评审问题清单（12 问）：见 docs/architecture/review/REVIEW-INVITE.md
- 裁决口径：采纳 / 有条件采纳 / 拒绝 / 延后，每项记录理由与验证条件
- 各 ADR 见 docs/architecture/adr/（ADR-NEW-14 至 ADR-NEW-23）
- 实测证据见 docs/benchmarks/（Spike A/B/C、存档、clippy、依赖验证报告）

请在对应子 issue（#1–#10）下按模板回复。
```

### #1 ADR-NEW-14 预算与降级必须确定性

**标题**：`[ADR-NEW-14] 预算与降级必须确定性——裁决` 
**正文**：

```
状态：Proposed（裁决后记入 docs/architecture/decisions/decision-log.md）
文件：docs/architecture/adr/ADR-NEW-14-预算与降级必须确定性.md
关联：2.7 / EC-04 / EC-11 / Stage 2C；评审问题 #4 #10

决策要点：预算与降级触发只用确定性工作量单位（A* 节点展开数、队列长度、本 tick 实体数）；墙钟/CPU/帧率仅遥测；机器性能调整只在 tick 边界显式变更并入存档与回放；LOD 切换条件同为确定性信号（条 6，2026-11-18 评审补入）。

关注点：逐系统计量单位与每 tick 上限的设定方法；R2/R3 分层后门禁耗时（实测数据请记 docs/benchmarks/）。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #2 ADR-NEW-15 确定性边界定义

**标题**：`[ADR-NEW-15] 确定性边界定义——裁决`
**正文**：

```
状态：Proposed
文件：docs/architecture/adr/ADR-NEW-15-确定性边界定义.md
关联：2.9 / 2.4 / EC-05 / EC-12；评审问题 #7

决策要点：2.9 八条契约（范围/权威状态划分/浮点禁令含位级确定性豁免/迭代顺序/显式顺序/并行快照+稳定键/u64 tick/命令生效 tick）。clippy disallowed-types/methods 分层已于 2026-11-18 实测可行（docs/benchmarks/clippy-layering-REPORT.md：HashMap、f32 全路径、Instant/SystemTime 均有效抓取，#[allow] 豁免有效）。

关注点：哪些条款只能靠评审（语义级迭代依赖、第三方 crate、宏展开）；跨平台（R4）口径。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #3 ADR-NEW-16 存档最小集合扩充

**标题**：`[ADR-NEW-16] 存档最小集合扩充——裁决`
**正文**：

```
状态：Proposed
文件：docs/architecture/adr/ADR-NEW-16-存档最小集合扩充.md
关联：EC-03 / EC-11 / 2.3 / EC-02；评审问题 #6

决策要点：七类字段最小集（元数据/身份高水位/随机数/路网/在途任务/预算降级/校验摘要）；在途任务"保存并恢复"已定案（强读档等价测试在丢弃策略下数学上不可能通过）。

实证：spikes/save-restore-spike 9/9 测试，含强读档等价（N=40 存档读档跑至 100 == 直接跑至 100），docs/benchmarks/SaveRestore-Validation-REPORT.md。

关注点：确定性摘要应覆盖哪些权威状态；永久兼容边界。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #4 ADR-NEW-17 路网事务的过期结果策略

**标题**：`[ADR-NEW-17] 过期寻路结果策略——裁决`
**正文**：

```
状态：Proposed
文件：docs/architecture/adr/ADR-NEW-17-路网事务的过期结果策略.md
关联：EC-01 / 2.5 / 2.6；评审问题 #3 #4

决策要点：请求/结果带 base_network_version；影响区域相交判断（含聚合规则：区间内全部事务影响区域并集；影响区域覆盖拓扑+动态代价）；只保证合法性不保证最优性；前向进展不变量 + 重算上限 + 兜底三候选（局部可行路径/沿用旧路径有效段/标记受阻）。

实证：spikes/network-tx-spike 8/8 测试（原子发布/聚合规则/合法性/前向进展/并发交错）、5395 ns/publish，docs/benchmarks/Spike-C-NetworkTx-REPORT.md；细则定案建议 ADR-NEW-23。

关注点：频繁修路场景的负载与活锁风险如何量化（重算上限、统计口径）。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #5 ADR-NEW-18 仿真参数基线与数值契约补充

**标题**：`[ADR-NEW-18] 参数基线与数值契约补充——裁决`
**正文**：

```
状态：Proposed（参数值已由项目负责人 2026-11-18 定案，参照 Cities: Skylines 2 同类数值）
文件：docs/architecture/adr/ADR-NEW-18-仿真参数基线与数值契约补充.md
关联：2.10 / 2.4 / EC-06 / Stage 0 / Spike A；评审问题 #1

定案值：tick 30 Hz / 地图 16 km×16 km / 分辨率 1 mm / 50 m·s⁻¹、10 m·s⁻² / 经济 10¹² 独立 i64 / 参考场景 100,000 Agent×1,000 Tick（R2 门禁 10k×1k 不变）/ 最大持续 10⁹ ticks（≈386 天）。

关注点：数值是否满足长期模拟要求（dt 漂移 1.6 mm/1e6 ticks，数据 docs/benchmarks/tick-rate-drift-output.txt）。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #6 ADR-NEW-19 GlobalDigest 摘要与规范化序列化

**标题**：`[ADR-NEW-19] GlobalDigest 与规范化序列化——裁决`
**正文**：

```
状态：Proposed（源自设计审计阻塞项 #1）
文件：docs/architecture/adr/ADR-NEW-19-GlobalDigest摘要与规范化序列化.md
关联：EC-05 / EC-03 / EC-11；评审问题 #6

决策要点：BLAKE3-256（备选 xxh3-64）；canonical 小端序列化（定长整数、u64 长度前缀、浮点禁入、与存档共用编码器）；digest_version u16；存档点全量、每 tick 滚动仅优化。

现状：save-restore-spike 等价测试用 FNV 占位（唯一替换点），裁决后换 BLAKE3。

关注点：确定性摘要应覆盖哪些权威状态（候选模块清单见 ADR）。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #7 ADR-NEW-20 请求/事务 ID 分配与在途寻路任务口径

**标题**：`[ADR-NEW-20] 请求/事务 ID 与在途寻路口径——裁决`
**正文**：

```
状态：Proposed（源自设计审计阻塞项 #2/#3）
文件：docs/architecture/adr/ADR-NEW-20-请求事务ID分配与在途寻路任务口径.md
关联：2.3 / EC-03 / 2.8 / ADR-NEW-16；评审问题 #6

决策要点：请求/事务 ID u64 单调分配器、高水位入存档最小集；在途寻路=单 tick 原子搜索（tick 边界无部分搜索状态→强读档等价天然成立）；中止尝试预算照常计入；备选（存档含部分搜索状态/预算幂等）及否决理由见 ADR。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #8 ADR-NEW-21 SimScalar 格式定案

**标题**：`[ADR-NEW-21] SimScalar=Q32.32 定案——裁决`
**正文**：

```
状态：Proposed（Spike A 收尾定案建议）
文件：docs/architecture/adr/ADR-NEW-21-SimScalar格式定案.md
关联：2.4 / EC-06 / Spike A；评审问题 #1

决策要点：Q32.32(i64)；舍入=就近、0.5 远离零；乘除中间 i128；权威时间 u64 tick；经济/里程独立 i64/u64；dt=1/30 from_ratio（黄金 dt_raw=143,165,577）。

实证：Rust 12 测试全过（漂移 +7 raw/tick @30 TPS）、定案核对 9 项硬约束 PASS、2.5 ns/op；否决 Q24.40（里程溢出）/Q48.16（漂移 45.8 m）/Q16.48（扩展余量不足）。
证据：docs/benchmarks/Spike-A-SimScalar-REPORT.md、simscalar-formal-check-output.txt。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #9 ADR-NEW-22 系统级 RNG 协议

**标题**：`[ADR-NEW-22] 系统级 RNG 协议——裁决`
**正文**：

```
状态：Proposed（Spike B 派生）
文件：docs/architecture/adr/ADR-NEW-22-系统级RNG协议.md
关联：EC-02 / EC-03；评审问题 #2

决策要点：双模式（纯派生默认零状态 + Stream64 顺序流 16 B 存档）；SplitMix64 为协议 v1 算法（备选 xoshiro256**/PCG64）；派生键编码 FNV-1a+两轮混合属协议内容；RNG_PROTOCOL_VERSION 入存档 bump 规则。

实证：spikes/rng-spike 9/9（流隔离/调度顺序无关/黄金复现）；SplitMix64 0.59 ns/draw（docs/benchmarks/rng-algo-comparison.txt）。

关注点：隔离、跨平台一致性（R4 未验）、版本兼容。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

### #10 ADR-NEW-23 路网事务过期结果细则

**标题**：`[ADR-NEW-23] 过期结果细则（相交粒度/重算上限/兜底/行走删路）——裁决`
**正文**：

```
状态：Proposed（Spike C 派生，为 ADR-NEW-17 待决事项提供实测定案建议）
文件：docs/architecture/adr/ADR-NEW-23-路网事务过期结果细则.md
关联：EC-01 / EC-07 / ADR-NEW-17 / ADR-NEW-20；评审问题 #3 #4

决策要点：影响区域=轴对齐包围盒（1 mm 网格）；聚合=区间并集；recompute_limit=3；超限兜底默认 MarkBlocked（三候选比选表见 ADR）；行走删路 RerouteNearest 优先、Stop 保底；处理顺序按请求 ID 稳定序。

实证：spikes/network-tx-spike 8/8（含发布/查询交错并发测试、walk_policy 三方案比选、前向进展），docs/benchmarks/Spike-C-NetworkTx-REPORT.md。

关注点：频繁修路场景负载与活锁的量化口径。

## 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：
- 验证条件：
```

---

## 自动建 issue（可选）

安装 [GitHub CLI](https://cli.github.com/) 并 `gh auth login` 后运行：

```powershell
pwsh docs/architecture/review/create-issues.ps1
```

脚本按本文件的 11 条草稿在 2222BUG/CitySim-rs 批量建 issue。
