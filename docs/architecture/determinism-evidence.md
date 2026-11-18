# 2.9 确定性边界八条契约证据表（Stage 1 T9）

- 日期：2026-11-19
- 依据：主文档 2.9（确定性边界唯一定义）；EC-12 契约即代码；EC-11 确定性回归
- 用法：每条契约的落实位置与证据（测试/检查）；R2 门禁启用前逐行核对。状态词表口径：
  ✅=已落实有证据；⚠️=骨架/部分落实，实施前补齐（对应债务/ADR）。

| # | 契约（2.9） | 落实位置 | 证据 | 状态 |
| --- | --- | --- | --- | --- |
| 1 | 范围：同一二进制同一平台逐 tick 可复现（跨平台列 R4） | 全部权威 crate；黄金值测试 | sim-save `strong_readback_equivalence`/`weak_form_equivalence`；sim-digest `golden_vectors`；sim-rng `reproducibility_golden` | ✅（跨平台 R4 待验） |
| 2 | 权威/非权威状态划分 | docs/architecture/crate-authority-list.md | .github/scripts/dep_graph_check.py（CI dep-graph job，规则 1-5） | ✅ |
| 3 | 权威系统不用平台 f32/f64（浮点仅渲染/UI/遥测） | 各权威 crate clippy.toml disallowed-types f32/f64 | clippy 分层实测（docs/benchmarks/clippy-layering-REPORT.md）；sim-codec 无浮点方法 | ✅ |
| 4 | 不依赖无序迭代（HashMap/HashSet/Query 顺序），按稳定键排序 | 权威 crate 用 BTreeMap/显式 sort（sim-rng NamespaceRegistry、sim-agent AgentRegistry、sim-save 队列 FIFO） | clippy disallowed HashMap/HashSet；sim-core `due_order_is_stable_regardless_of_storage_order` | ✅（语义级依赖靠评审+属性测试，ADR-NEW-15 条 4 口径） |
| 5 | 权威系统显式 before/after（全序提交与执行顺序分离） | sim-core run_tick 固定相位序（发布→处理→游走→降级→产生） | 骨架期以固定相位代替调度器；全序键候选 (phase_id, system_id, request_id, agent_id, local_sequence) 见 ADR-NEW-15 条 5（P1-02 修订） | ⚠️（调度器落地=Stage 2B） |
| 6 | 并行只读 tick 快照 + 稳定键排序串行提交 | 未实装（Stage 2B 范围） | 参考：spikes/network-tx-spike `interleaved_publish_query_threads` | ⚠️（Stage 2B） |
| 7 | 权威时间 u64 tick，不读系统时钟 | sim-core SimClock（advance 唯一入口）；sim-save in_tick 相位 | clippy disallowed Instant::now/SystemTime::now；sim-save `save_rejects_non_tick_boundary` | ✅ |
| 8 | 玩家命令带生效 tick 并记录供回放 | sim-core CommandLog（effective_tick + 全序 + ID 高水位 + from_parts 重建） | sim-core 7 测试（含 due 全序、过期拒绝） | ✅ |

## R2 场景骨架（本任务交付）

- 载体：`crates/sim-save/tests/r2_scenario.rs`（sim-test-support SimHarness 适配 sim-save）。
- 已含：同种子两跑摘要一致（repeat_run_digests）、强读档等价（strong_readback_digests）、
  smoke fixture 驱动。摘要折叠为 u64 仅适配 SimHarness 接口；R2 oracle 用完整
  256 位 BLAKE3（sim-save `state_digest`）。
- 启用 R2 门禁（CI `r2-deterministic` 占位 job）前置：模拟核心完整 tick 循环
  （T9 之后=Stage 2 范围）、R2 fixture 全量 10,000 Agent × 1,000 Tick 跑通、
  GlobalDigest 黄金向量比对接入（docs/benchmarks/globaldigest-golden-vectors.txt）。

## 逐条核对结论（2026-11-19，T9）

八条中 6 条 ✅ 有代码与测试证据；#5/#6 标 ⚠️（调度器与并行提交属 Stage 2B，
骨架期以固定相位序+单线程代替，契约文字已由 ADR-NEW-15 P1-02 修订钉全序键口径）。
