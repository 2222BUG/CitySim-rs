# ADR-NEW-21：SimScalar 数值格式定案（Spike A 数值设计决策）

- 编号：ADR-NEW-21（暂定编号，正式入仓时按仓库 ADR 序列重新编号并保留映射）
- 状态：Proposed
- 日期：2026-11-18
- 关联主文档章节：2.4（数值系统 SimScalar）、2.10（仿真参数基线）、2.9（确定性边界）、Spike A、Stage 0
- 来源：Spike A 收尾（Step 2，2026-11-18）；前置 ADR-NEW-18（参数基线已定案）

## 背景

2.10 参数基线已于 2026-11-18 由项目负责人定案（Cities: Skylines 2 参考值：tick 30 Hz、地图 16 km × 16 km、分辨率 1 mm、最大速度 50 m/s、加速度 10 m/s²、经济 10^12、参考场景 100,000 Agent、最大持续 10^9 ticks）。Spike A 已产出 Python 模拟（analysis.py）、Rust 原型（spikes/simscalar-spike，12 测试全过）与定案参数正式核对（docs/benchmarks/simscalar-formal-check.py，9 项硬约束全 PASS）。据此可对 SimScalar 格式作出数值设计决策。

## 问题

Q32.32、Q24.40（及 Q48.16、Q16.48）中哪个作为 SimScalar 格式？舍入规则与中间值位宽如何定？

## 决策

1. **SimScalar 采用 Q32.32（i64：32 位整数 + 32 位小数）**，即 `SimScalar(i64)`，LSB = 2^-32 m ≈ 2.33e-10 m，整数范围 ±2^31 m。
2. **舍入规则**：乘除采用"就近舍入，0.5 向远离零取整"（ties away from zero）；截断被否决（误差单向累积，analysis.py 证据）。
3. **乘除中间值用 i128**：50 m/s × dt 需 66 bit（formal-check [4]），i128 足够且留有余量；溢出走 checked_* / saturating_*，不静默环绕。
4. **权威时间用 u64 tick**，不用 SimScalar 秒累积（Q32.32 秒累积约 68 年溢出，ADR-NEW-18 数值契约 1）。
5. **经济数值（10^12）、里程/统计类无界累积字段用独立 i64/u64 整数类型**，不经 SimScalar（ADR-NEW-18 数值契约 4；formal-check [5][6]）。
6. **dt = 1/30 s 由 from_ratio 一次派生**，黄金值 dt_raw = 143,165,577（舍入误差 +1.087e-10 s 固定），写入黄金测试。

## 考虑过的替代方案

| 方案 | 评价 |
| --- | --- |
| Q24.40 | 精度更高（漂移 0.006 m/1e9 ticks），但 10^9 ticks 最坏里程 1.667e9 m 超出其整数位 ±2^23≈8.39e6 m（2026-11-19 勘误：原文"±1.678e7 m"系 2^24 无符号口径；Q24.40 整数位含符号应为 ±2^23≈8.39e6 m，与"0.3 年秒溢出"一致；否决结论不变），里程类字段将被迫独立类型；且格式换档收益边际，否决 |
| Q48.16 | dt 舍入误差大，漂移 +45.8 m/1e6 ticks，否决（analysis.py） |
| Q16.48 | 整数位 ±32,768 m 恰容纳 16 km 地图（半幅 8 km 富余 4 倍），但排除任何 > 65.5 km 宽的扩展；此前"排除 64 km+ 地图"的否决理由不准确（见修订记录），以扩展余量不足为由否决 |
| 先定格式再定参数（历史路线） | 已被 ADR-NEW-18 否决 |

## 影响

- 主文档 2.4 的"Q32.32 / Q24.40 候选"进入定案建议：Q32.32（本 ADR 为唯一格式裁决依据，正式裁决前 2.4 保持候选表述 + 本 ADR 注）。
- 2.9 确定性边界第 3 条（禁平台 f32/f64）由 SimScalar 定点路径落实；Rapier 仍限非权威侧（ADR-NEW-15）。
- dt 黄金测试（dt_30hz_from_ratio_golden、drift_golden_30tps）进入 EC-11 R2 黄金测试基线（不得自动覆盖）。

## 验证方法

- Python 侧：docs/benchmarks/simscalar-formal-check.py（9 项硬约束）+ analysis.py + tick-rate-drift.py（输出均入仓 docs/benchmarks/）。
- Rust 侧：spikes/simscalar-spike 12 测试全过（含 dt_30hz_from_ratio_golden、drift_golden_30tps 黄金值、intermediate_width_fits_i128、map_16km_and_1mm_fit）；性能冒烟 2.5 ns/op（release，非正式基准，docs/benchmarks/simscalar-perf.txt）。
- 跨平台（R4）发布前复验：黄金值不依赖平台浮点，预期一致，尚未实测。

## 证据范围声明

Rust 实测环境：rustc 1.100.0（0ef1bc386 2026-11-09），x86_64-pc-windows-msvc，单机。2.1 技术基线写作 Rust 1.89，差异（实测 1.100.0）待收敛记录；跨平台未验证（R4）；性能数据为非正式冒烟基准，不作门禁阈值。

## 修订记录

- （Spike A 早期评审，2026-11-18）Q16.48 否决理由修正：±32,768 m 恰容纳 16 km 地图，"排除 64 km+ 地图"表述不准确；否决理由改为扩展余量不足（详见 docs/benchmarks/Spike-A-SimScalar-REPORT.md 修订记录 P3）。

## 当前状态

Proposed。数值设计决策（Spike A 交付物），待正式评审裁决；裁决通过后 2.4 由"候选"改为"Accepted"并记录本 ADR 编号（重编号后同步映射）。


## 正式裁决（2026-11-19）

- 裁决人与依据：项目负责人委托五家独立 AI 评审（报告 A/B/C/D/E）汇总，维护者代录；五家共识详见 decision-log《2026-11-19 正式裁决记录》。第 5 家（E）裁决方向与共识一致，但其证据引用不实（虚构 GlobalDigest-test、ADR-12 等），证据断言已驳回，不作验证依据。
- 裁决：**采纳 → Accepted（无条件）**。
- 理由：证据最扎实（analysis.py+tick-rate-drift+12 测试+formal-check），五家中 A/C/E 直接采纳；Q24.40 整数位数字已勘误（±2^23≈8.39e6 m），否决结论不变。
- 转 Verified 的外围条件：工具链收敛（债务 #22）、正式基准替代冒烟、dt/漂移黄金值入 R4 门禁。
