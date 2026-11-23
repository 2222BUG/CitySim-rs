# criterion 正式基准基线（2026-11-23，O1）

- 采纳：criterion 0.5（dev-deps，开源采纳 O1）——正式基准设施，替代自写冒烟计时。
- 环境：rustc 1.100.0 / Windows x86_64 / release（bench profile）；墙钟仅遥测口径
  （2.9），确定性以展开计数与逐位一致测试为准。
- 场景：200×200 双向网格（1mm 网格、边代价 1000、40k 节点、79.6k 边）——与 G5 系列同构。
- 样本：criterion 默认统计，sample_size=10。

| 基准 | 时间（中位数 [min, max] 估计区间） |
| --- | --- |
| astar_200x200/corner_to_corner（对角最坏请求，预算=|V|） | 248.31 µs [247.18, 249.23] |
| astar_200x200/mid_range（中距请求） | 70.35 µs [69.915, 70.735] |

## 报告口径

- 复现：`cargo bench -p sim-pathfinding --bench astar_bench`；criterion 自带历史对比
  （change 行），回归判定用其 change 区间，不另设自算阈值。
- 本基线为首个正式基线（此前 2.5 ns 冒烟不得用作门禁阈值——决策5 条件）。
- 后续扩展：sim-engine run_tick 吞吐基准、R2 场景端到端基准（随 R2 门禁启用）。
