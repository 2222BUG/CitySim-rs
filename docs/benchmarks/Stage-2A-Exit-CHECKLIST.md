# Stage 2A 退出核对（2026-11-19）

对照 ADR-NEW-24 决策 4「Stage 2A 退出判据表」逐条核对。证据均为 rustc 1.100.0 Windows x86_64 实测。

| # | 判据 | 阈值 | 实测 | 证据 | 判定 |
| --- | --- | --- | --- | --- | --- |
| 1 | 最优性对拍（A* vs 参考 Dijkstra） | 代价全等 | 5×5 网格 4 组全等 | sim-pathfinding 测试 astar_matches_dijkstra | ✅ |
| 2 | 结果唯一性（同输入同路径） | 唯一 | (f,h,node_id) 全序；2×2 等价路黄金断言 path==[1,3] | astar_is_unique、tie_break_is_deterministic | ✅ |
| 3 | 预算让步解析上界 | k ≤ ⌈log₂(\|V\|/b)⌉+1 | 上界 11，实测 4 ticks | budget_yield_respects_analytic_bound；G5-stress-200x200.txt | ✅ |
| 4 | MarkBlocked 阻塞率 | ≤ 1% | 0.00（churn 200 请求全终态） | G5-churn-200x200.txt | ✅ |
| 5 | 过期重算率（churn） | ≤ 5% | 0.0027（3/1100） | G5-churn-200x200.txt | ✅ |

补充实测（G1-G5 组级验收，机制 A/B/C 对比）：G5-Verdict-REPORT.md——B=4 ticks/浪费 0.53/
最坏单 tick 0.139 ms 无尖峰；C 单独不可用；A 零浪费高延迟，保留切换路径。

## 结论

- **Stage 2A 实施面退出条件 5/5 满足**（附双平台未验、单线程未验、并发队列压测随 Stage 2B 的限定）。
- **ADR-NEW-24 仍为 Proposed**：定案（CSR/A*/机制 B 参数/退出判据）需按第 7 章流程正式裁决
  （或负责人委托裁决）后转 Accepted；裁决前 ADR-NEW-24 的"定案"表述一律按 Proposed 口径阅读。
- 遗留（转 Stage 2B/后续门禁）：并发队列压测、让步路径强读档等价专项、Edge 有向语义入数据字典
  （债务 #8 增补）、机制 B 重复展开率阈值定案（实测 0.53 为讨论数据，G2 待正式锁定）、R3/R4。

## 复现

```
cargo run --release --example g5_stress -p sim-pathfinding   # 机制 A/B/C 对比
cargo run --release --example g5_churn -p sim-pathfinding    # churn 重算率
cargo test --workspace                                       # 140+ 测试
```
