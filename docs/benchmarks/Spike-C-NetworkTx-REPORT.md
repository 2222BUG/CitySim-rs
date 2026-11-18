# Spike C：路网事务原子发布 — 初步报告（Spike 完成版）

- 日期：2026-11-18　状态：**Spike 已完成；结论为定案建议（Proposed，待评审裁决）**
- 原型：`spikes/network-tx-spike/`（零依赖、`#![forbid(unsafe_code)]`、rustc 1.100.0）
- 对应：主文档第 8 章 Spike C；ADR-NEW-17（过期结果策略）待决事项取数；ADR-NEW-20（单 tick 原子搜索）衔接
- 证据纪律：单机原型验证；确定性寻路为 BFS+稳定键（正式 A* 确定性细则属决策债务 #17）；
  性能为冒烟参考值（见 `docs/benchmarks/network-tx-perf.txt`），非正式基准。

## 一、主文档六项验证结果

| 验证项 | 结果 | 证据 |
| --- | --- | --- |
| 路网后台重建 / 事务边界 | ✅ | `atomic_publish_and_versioning`：Tx 失败整体不生效、version 不变、无部分生效 |
| 原子发布 | ✅ | 同上；`interleaved_publish_query_threads`：20 笔事务跨线程发布恰好 version==20 |
| 查询版本一致性 | ✅ | `query_consistency_and_accept`：结果携带 base_version；不相交区间过期结果被接受 |
| 取消与失败 | ✅ | `abort()`（临时副本验证，权威状态零改动）；失败事务 Err + 状态回滚 |
| 并发冲突（发布与查询交错） | ✅ | `interleaved_publish_query_threads`：1 发布 + 2 查询线程不变量成立 |
| 过期结果统计 | ✅ | `stale_aggregation_rule` / `stale_legality_check` / `forward_progress_under_churn`：Accept/Requeue/DropFallback 三路统计（Stats 结构） |

## 二、ADR-NEW-17 待决事项取数（定案建议）

| 待决事项 | 实测定案建议 | 依据 |
| --- | --- | --- |
| 影响区域表示法（债务 #9） | 轴对齐包围盒 Region{i64×4}，网格单位 1 mm（2.10 分辨率）；拓扑修改与动态代价更新都声明影响区域 | 条文 8 覆盖两类；包围盒求交 O(1)，保守相交=宁可重算不误接受 |
| 相交粒度 | 区间内全部事务影响区域**并集** × 路径区域（路径边几何包围盒并集） | `stale_aggregation_rule`：多笔事务区间须按聚合规则判断 |
| 重算上限 | 每请求 recompute_limit = 3（超限进入兜底） | `forward_progress_under_churn`：持续修路 10 tick 内必达终态 |
| 兜底三选一 | **默认标记受阻（MarkBlocked）**；局部可行路径（PartialPath）/沿用有效段（UseValidPrefix）为后续增强，枚举已预留 | 受阻是唯一无歧义终态；另两个需定义"部分路径可接受性"（债务 #18 相关） |
| 行走中道路被删除 | **RerouteNearest 优先（就近重算），不可达时进兜底；Stop 为安全保底** | `walk_policy_comparison` 对比（见下表） |
| 处理顺序 | 按请求 ID 稳定排序（BTreeMap pending） | 2.9 条 4 稳定序要求 |

### 行走删路三方案比选（walk_policy_comparison 实测语义）

| 方案 | 行为 | 优点 | 缺点 |
| --- | --- | --- | --- |
| Stop（原地停止） | 冻结在当前节点 | 永远合法、零开销 | 永久停滞，需外部触发恢复 |
| Backtrack（回退） | 退到已走有效段起点 | 路径永远合法 | 长路径退化代价大；仍未前进 |
| RerouteNearest（就近重算） | 当前节点重路由；不可达→兜底 | 唯一持续前进的方案 | 有重算开销；依赖兜底处理不可达 |

## 三、过期统计（原型内 Stats，供 EC-01 验证判据参考）

- 合法 + 不相交 → Accept（`stale_accepted`）；相交或边被删 → Requeue（`stale_requeued`）；
  超 recompute_limit → DropFallback（`stale_dropped`）。
- 前向进展不变量（ADR-NEW-17 条文 10）：**每个请求在有限 tick 内获得路径或进入受阻兜底**——
  `forward_progress_under_churn` 在持续每 tick 修路下验证成立。
- 合法性（条文 9）单独成测：路径边被删必 Requeue，与影响区域判断解耦。

## 四、性能（冒烟，release）

- 5395 ns/publish（混合均值，含每 10 笔一次 200 边 BFS 查询 + 过期判定）；
- 原型 Tx::apply 用 clone+提交（正确性优先）；正式实现换双缓冲后发布成本可降一个数量级以上（Stage 2A/2C 优化项）。

## 五、遗留

1. 真实路网规模（200×200 网格、40k 节点）的发布/查询吞吐未测——归 Stage 2A 基准（docs/benchmarks/）。
2. 过期结果统计缺"固定场景下数量与开销"完整数据（主文档 EC-01 验证判据仍缺阈值，债务 #18）。
3. 路网数据字典（节点/边 ID 语义、动态代价存储）仍是债务 #8；本原型 Edge 为最小可行结构。
4. 取消语义的并发撤销（发布中事务被打断）未覆盖——正式版需事务日志回滚点（随存档序列化 ADR-19 编码器一并定）。
5. BFS 确定性依赖 BTreeMap 稳定序；正式 A* 平局打破/邻居序细则属债务 #17。
