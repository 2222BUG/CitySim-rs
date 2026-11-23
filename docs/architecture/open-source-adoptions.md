# 开源方案采纳与对接登记

- 目的：周边层（基准/测试/可视化/遥测/内容管线）优先采用成熟开源件，自研聚焦确定性内核。
- 选型核对线（五条）：许可 MIT/Apache、MSRV ≤1.89（ordered-float 教训）、Bevy 0.18
  基线相容、权威侧三禁（f32/f64、HashMap/HashSet 迭代顺序、Instant/SystemTime）、
  依赖方向规则（dep_graph_check 规则 1-5 + fail-closed）。
- **红线**：确定性核心协议（SimScalar Q32.32、canonical 编码、GlobalDigest、RNG 协议、
  稳定键调度）由黄金值+多轮外部审计锁定，不以任何第三方实现替换。

## 已采纳登记

| 方案 | 版本 | 落位 | 许可 | 对接点 | 状态 |
| --- | --- | --- | --- | --- | --- |
| criterion | 0.5 | sim-pathfinding dev-deps（benches/astar_bench） | MIT/Apache | 报告口径=criterion 统计入 docs/benchmarks/criterion-*.md；回归判定用 change 区间 | O1 ✅ 2026-11-23 |
| proptest | 1 | sim-network/sim-pathfinding dev-deps（tests/proptest_*） | MIT/Apache | 属性=发布原子性/过期合法性/预算上界；失败输入落 proptest-regressions/ 提交回放 | O1 ✅ 2026-11-23 |

## 计划采纳（待落地）

| 方案 | 落位 | 对接点 | 阶段 |
| --- | --- | --- | --- |
| insta | dev-deps（黄金快照） | 替换裸 assert 黄金值（Digest256 hex 快照） | O1 可选 |
| bevy_ecs_tilemap 0.18.1 | sim-view（非权威） | 渲染网格+Agent 状态，只读对接 Engine::snapshot | O2 ✅ 2026-11-23（编译+6s 冒烟） |
| egui_plot 0.34 / bevy_egui 0.39 | sim-view | 曲线面板（treasury/agents）；egui_plot 0.34↔egui 0.33 版本对齐（0.33 会锁双 egui） | O2 ✅ 2026-11-23 |
| bevy-inspector-egui | sim-view | 调试检查器（暂缓：版本对齐待核，不阻塞 Demo） | O2 可选 |
| puffin 0.20 / puffin_http 0.17 | sim-view demo（非权威遥测） | profile_scope 采样+本地 viewer 服务；墙钟仅遥测豁免口径 | O3 ✅ 2026-11-23 |
| cargo-fuzz（libfuzzer-sys 0.4） | fuzz/（独立布局，workspace exclude） | save_load/replay_decode 解码拒绝路径；nightly 专项，CI 不跑，字节汤日常回归=proptest fuzz_decode.rs | O3 ✅ 2026-11-23 |
| osm2streets | 离线管线 | OSM→中间路网，产物过确定性校验（稳定 ID+CSR 重建+digest）后准入 | 内容阶段 |
| criterion（扩展） | sim-engine | run_tick 吞吐基准 | 随 O1 扩展 |

## 明确不采纳（冲突）

rayon/par_iter（提交序不确定）、serde+bincode 权威序列化（canonical 协议）、rapier 权威
物理（f32）、fixed crate 替 SimScalar（协议锁定）、SUMO（GPL/EPL，仅外部对照 oracle）、
Bevy TaskPool 权威调度（顺序不可控）。
