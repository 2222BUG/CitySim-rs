# Spike B: 系统级 RNG — 报告（Step 3）

- 日期：2026-11-18
- 状态：Proposed（Spike 原型结论，待评审）
- 原型：`spikes/rng-spike/`（零依赖，`#![forbid(unsafe_code)]`，9 测试全过）
- 证据：`docs/benchmarks/rng-algo-comparison.txt`（速度冒烟）；黄金值固化于 `reproducibility_golden` 测试
- 实测环境：rustc 1.100.0，x86_64-pc-windows-msvc，单机；跨平台未验证（R4）

## 验证结果（对主文档 Spike B 六项）

| 验证项 | 结果 | 证据 |
| --- | --- | --- |
| 系统随机流隔离 | ✅ 成立 | `system_stream_isolation`：traffic 消耗 10 万次不改变 economy 序列；不同命名空间同键不同值 |
| Agent 随机流隔离 | ✅ 成立 | `agent_stream_isolation`：agent 2 消耗 5 万次不改变 agent 1 序列 |
| 调度顺序变化的影响 | ✅ 无影响 | `schedule_order_independence`：A/B 交错、倒序执行，各流序列不变（纯派生天然成立；顺序流按流分组同样成立） |
| 可复现性 | ✅ 成立 | `reproducibility_golden`：黄金值固化（derive_seed / event_u64 / Stream64 首值），同种子重放一致 |
| 随机协议版本化 | ✅ 机制就绪 | `protocol_version_pinned`：RNG_PROTOCOL_VERSION=1 钉死；变更必须 bump 并修订 ADR-NEW-22 |
| 派生式 vs 有状态流存档差异 | ✅ 已量化 | `save_restore_and_state_footprint`：纯派生零状态存档（只需全局种子+协议版本）；有状态流 16 字节/流（seed+counter），保存-恢复续列一致 |

## 算法比较（release 冒烟，1e7 draws，非正式基准）

| 算法 | ns/draw | 状态 | 评价 |
| --- | --- | --- | --- |
| SplitMix64 | 0.59 | 推荐 | 最快；无大状态（8 B）；适合派生与顺序流；统计质量冒烟通过（正式统计测试未做） |
| xoshiro256\*\* | 0.73 | 备选 | 速度相当；状态 32 B；跳跃函数生态好（本 spike 未实现跳跃） |
| PCG64 (XSL-RR 128) | 1.14 | 备选 | 质量口碑最好；u128 状态；速度 2 倍慢于 SplitMix64 |
| derive_seed（协议派生） | 0.20 | - | 派生成本可忽略 |
| event_u64（纯派生抽取） | 0.25 | - | 每事件抽取成本可忽略 |

结论：三候选速度全部远低于预算（10 万 Agent 每 tick 各抽 1 次 ≈ 0.025 ms）；推荐 **SplitMix64** 作为协议 v1 算法（派生+顺序流），备选 xoshiro256\*\*。

## 推荐设计（待 ADR-NEW-22 裁决）

1. **双模式**：事件驱动行为用纯派生 `event_u64(global_seed, namespace, event_key, draw_index)`（零状态、调度无关）；顺序消费场景用 `Stream64`（seed+counter，16 B 存档）。
2. **派生键编码**：namespace UTF-8 → FNV-1a 64；mix(global_seed ^ ns_hash, event_key) 两轮 SplitMix64 混合；编码顺序固定属协议内容。
3. **版本化**：RNG_PROTOCOL_VERSION 入 EC-03 存档；算法/编码/输出映射变更必须 bump。

## 遗留（非阻塞）

- 统计质量正式测试（TestU01/PractRand 级）未做——Spike 范围外，定案前如需可补卡方/序列测试。
- 跨平台黄金值复验（R4 发布前）。
- 事件键编码中 namespace 的注册表（系统命名空间清单）随 decision-log 债务 #15（crate 权威/非权威清单）一起定。
