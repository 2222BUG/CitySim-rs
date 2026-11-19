# AI 评审任务书：ADR-NEW-24 裁决（Stage 2A 实施验证完毕）

> 你是独立评审人。请对 CitySim-rs 的 **ADR-NEW-24（路网 CSR 与确定性 A* 细则）** 给出正式裁决。
> 只依据本任务书与所附材料原文判断；不确定的地方明说，**"拒绝"和"延后"是合法裁决**。

## 1. 项目背景（30 秒）

CitySim-rs = 基于 Rust + Bevy 的开源城市模拟（设计评审/早期实施阶段），以**确定性**为核心承诺
（同种子同输入逐 tick 可复现、强读档等价）。设计文档 V1.9.3 + ADR 体系（ADR-NEW-14~23 已由
六家独立 AI 评审共识裁决转 Accepted；ADR-NEW-24 是 Stage 2A 定案前置的新草案）。
仓库：https://github.com/2222BUG/CitySim-rs

## 2. 被评审对象

**ADR-NEW-24-路网CSR与确定性A星细则.md**（状态：Proposed）
https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/adr/ADR-NEW-24-%E8%B7%AF%E7%BD%91CSR%E4%B8%8E%E7%A1%AE%E5%AE%9A%E6%80%A7A%E6%98%9F%E7%BB%86%E5%88%99.md

四项决策摘要：

| # | 决策 | 要点 |
| --- | --- | --- |
| 1 | CSR 路网 | offsets Vec<u32> + adj_edge_ids 按 (node_id,edge_id) 升序；拓扑事务整表重建原子替换；代价变更原位更新 |
| 2 | A* 确定性五条 | 曼哈顿定点启发式可采纳；open 排序键 (f,h,node_id) 全序；邻居展开按 CSR 数组序；closed 首展闭合、g 严格更小才 reopen；禁 BinaryHeap |
| 3 | 机制 B 参数 | g=2、L=\|V\|、B_k=min(b·2^k,L)；解析上界 k≤⌈log₂(\|V\|/b)⌉+1；累计浪费≤2\|V\|；不触发 ADR-NEW-14 条 4 配置变更 |
| 4 | Stage 2A 退出判据 | 最优性对拍/唯一性/重算率≤5%/阻塞率≤1%/预算上界 + 取消语义（队列移除、ID 不复用）+ 冲突合并（tx_id 全序串行） |

否决项：BinaryHeap（无全序）、Dijkstra 全量（无启发式浪费）、浮点启发式（违 2.9）、增量拓扑（暂缓待 G5 数据）。

## 3. 实施证据（已实测，全部可复现）

| 证据 | raw 链接 |
| --- | --- |
| G5 机制 A/B/C 压测报告 | https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/G5-Verdict-REPORT.md |
| Stage 2A 退出核对（判据 5/5） | https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/Stage-2A-Exit-CHECKLIST.md |
| G5 压测原始数据（200×200） | https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/G5-stress-200x200.txt |
| churn 重算率原始数据 | https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/benchmarks/G5-churn-200x200.txt |
| 决策台账（含债务表） | https://raw.githubusercontent.com/2222BUG/CitySim-rs/main/docs/architecture/decisions/decision-log.md |

关键实测数字：200×200 网格（40k 节点）角到角最长请求——机制 A=7 ticks 零浪费、
机制 B=4 ticks 重复展开率 0.53（解析上界 11 实测 4）、机制 C=100% 阻塞；churn 场景
200 请求+10 笔拓扑事务：重算率 0.0027、阻塞率 0.00、最坏单 tick 0.139 ms 无尖峰。
实现位于 crates/sim-network 与 crates/sim-pathfinding（16 测试，含 Dijkstra 最优性对拍）。

## 4. 请重点裁决的问题

- **Q1**：A* 平局打破口径 (f,h,node_id)、h 升序优先最深前沿——接受吗？（决定路径形态与展开量；
  实测该口径下角到角仅展开 ~398 节点，无全域能退化。）
- **Q2**：机制 B 参数（g=2/L=|V|）与重复展开率阈值——实测 0.53；切换到机制 A 的阈值定多少？
- **Q3**：Edge 有向语义（双向通行=建两条边）——写入数据字典（债务 #8）的口径确认。
- **Q4**：退出判据五条（含 churn 重算率≤5%、阻塞率≤1%）是否足以支撑 Stage 2A 收口？
- **Q5**：与 ADR-NEW-20（预算让步）/NEW-23（过期结果细则）的联动一致性有无遗漏？

## 5. 裁决格式（请原样返回）

```
## ADR-NEW-24 裁决记录
- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后
- 理由：（1-3 句）
- 验证条件 / 修改要求：（逐条）
- 对 Q1-Q5 的逐项意见：
- 新增风险（如有）：
```

## 6. 注意事项

1. 证据纪律：只依据链接材料原文；材料未覆盖的点标"未见证据"，不要脑补。
2. 状态词表：ADR 现为 **Proposed**；裁决后由维护者记入 decision-log 并迁移状态。
3. 拒绝合法：如果认为某决策实测证据不足或方向错误，明确"拒绝/延后"并给出补证要求。
4. 联动组：ADR-NEW-17/20/23 已裁决为联动组（G1-G5 验收）；本 ADR 是其寻路侧实施定案，一致性意见请点名到条文。
5. 回传：回复到 GitHub issue #24（https://github.com/2222BUG/CitySim-rs/issues/24），
   或把裁决记录文本直接发给委托人转交。
6. 署名：请在报告中注明真实名字 / GitHub 昵称 / AI 模型具体型号（三选一，可并列；AI 署名须到具体型号/版本，泛称视同未署名）；
   未署名报告不计入裁决统计。
7. 代码级审查：请至少安排一位评审人做代码级审查（通读被评审模块源码与测试，不只看条款）；
   代码缺陷主张须给出文件+行号级证据（如 `lib.rs:182 wrapping_add`），未给出处按"未见证据"处理。

——维护者，2026-11-19
