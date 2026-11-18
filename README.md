# CitySim-rs

基于 Rust + Bevy 的开源城市模拟项目——技术方案与架构决策仓库。

> **状态：设计评审阶段（V1.9.3 公开评审稿）。** 本仓库当前仅包含技术方案、架构决策记录（ADR）与 Spike 数值原型（`spikes/simscalar-spike`，Stage 0 验证件，非功能实现），尚无实现代码。V1.9.3 新增条文均标记 **Proposed**；2.10 参数基线数值全部待定。按文档自身的证据纪律：设计目标、规划指标和 Spike 预期结果不得表述为已经实现的事实。

## 阅读入口

- [CitySim-rs 技术方案 V1.9.3](docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md)（唯一现行主文档）
- [决策日志](docs/architecture/decisions/decision-log.md)（裁决记录与待决事项）
- 架构决策记录：`docs/architecture/adr/`（ADR-NEW-14 ~ 18，五项修订提案；ADR-NEW-19 ~ 20，设计审计新增草案；均为暂定编号）
- 历史版本与评审原件（回退依据）：`docs/architecture/history/`
- 评审流转副本（docx）：`CitySim-rs技术方案V1.9.3.docx`
- ADR 文档包（分发件）：`CitySim-rs-ADR文档包.zip`

## 仓库结构

```
docs/
├─ architecture/
│  ├─ CitySim-rs-Technical-Spec-V1.9.3.md   # 唯一现行主文档
│  ├─ adr/                                  # 架构决策记录
│  ├─ decisions/decision-log.md             # 裁决与决策状态
│  ├─ testing/                              # 测试计划与测试规范
│  └─ history/                              # 历史版本与评审原件
├─ benchmarks/                              # 可复现实验脚本与基准结果（Spike A 证据、tick 频率漂移数据）
spikes/
└─ simscalar-spike/                         # Spike A：SimScalar Q32.32 定点原型（cargo test 7/7）
```

## 当前阶段与待办

- 五项修订提案（ADR-NEW-14 ~ 18）已草案并入正文（条文标 Proposed），整体裁决待正式评审；首轮评审 13 项修订意见的裁决记录见 decision-log。
- 设计审计（25 条发现）已完成：3 条阻塞项立项为 ADR-NEW-19（GlobalDigest）/ ADR-NEW-20（请求 ID 与在途任务口径），其余 20 项未登记决策债务已入账 decision-log。
- 2.10 参数基线表数值待项目负责人定案（Stage 0 前；每行已注明定案方法与截止点）。tick 频率漂移决策数据已入仓 `docs/benchmarks/tick-rate-drift.py`，可直接支撑第一行拍板。
- Spike A 证据已入仓 `docs/benchmarks/`（analysis.py 复核重建版 + 输出 + 修订版报告）；Spike A 原型入仓 `spikes/simscalar-spike/`（rustc 1.100.0 实测 7/7，文档基线 1.89 待收敛定案）。
- Stage 1 任务清单与原始路线图缺失，须重建入仓后方可开工 Stage 1（decision-log 债务登记 #19）。
- ADR-NEW-01 ~ 13 待建档（V1.9.3 正式版发布前）；ADR-NEW-14 ~ 20 为暂定编号，正式入仓裁决后重编号并保留映射。

## 许可证

MIT OR Apache-2.0 双许可。详见 `LICENSE-MIT` 与 `LICENSE-APACHE`。
