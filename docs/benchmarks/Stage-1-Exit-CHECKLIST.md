# Stage 1 退出条件核对表（2026-11-19）

- 依据：docs/architecture/Stage-1-Task-List.md 退出条件 6 条
- 执行日志：decision-log《Stage 1 执行日志》S1-1 ~ S1-10 全部完成
- 提交序列：ef6defa（S1-1 清单）→ eec339e（T1）→ 3ff0430（T2）→ d1bd0e1（T3）→
  43cd845（T4）→ 01498c4（T5）→ b19d41f（T6）→ f619178（T7）→ 7e2b0d0（T8）→
  fed0189（T9）

## 逐条核对

| # | 退出条件 | 结论 | 证据 |
| --- | --- | --- | --- |
| 1 | 可复现模拟循环（同种子两跑 digest 相等） | ✅ | crates/sim-save/tests/r2_scenario.rs `r2_same_seed_two_runs_agree`；sim-digest `order_independent`/`golden_vectors`；sim-rng `reproducibility_golden` |
| 2 | 强读档等价（存档读档跑至 M == 直接跑至 M） | ✅ | sim-save `strong_readback_equivalence`（N=40→M=100）；r2_scenario `r2_strong_readback_via_harness`（sim-test-support 模板） |
| 3 | Spike 黄金一致 | ✅ | T2 simscalar 12 测试（dt_raw=143,165,577、drift step_raw=2,147,483,655 与 Spike A 一致）；T5 sim-rng 黄金值 4 组与 Spike B 一致 |
| 4 | 2.9 八条契约证据 | ✅ | docs/architecture/determinism-evidence.md（6 条 ✅ 有代码/测试证据；#5 系统顺序、#6 并行提交标 ⚠️ 属 Stage 2B 范围，全序键口径已由 ADR-NEW-15 P1-02 修订钉定） |
| 5 | GlobalDigest 黄金向量 R2 oracle 可用 | ✅ | docs/benchmarks/globaldigest-golden-vectors.txt（3 组向量+纳入范围）；sim-digest `golden_vectors`；r2_scenario 摘要接入（oracle=完整 256 位 BLAKE3） |
| 6 | 条件核销清单对应项勾销 | ✅ | decision-log 条件核销清单：Stage 1 对应 5 项核销（NEW-15 分层落地、NEW-16 存档序列化格式、NEW-18 舍入 Rust 验证、NEW-19 编码器+黄金向量、NEW-22 namespace 注册表），累计 8/28；G3（让步状态入档入摘要）已落实并注记 |

## 总判定

**Stage 1 退出条件 6/6 满足（2026-11-19）**——T1~T9 全部交付并各自提交推送；
workspace 回归全绿（13 crate，58+ 测试；clippy 权威侧 CLEAN；依赖方向 PASS）。

## 遗留（转 Stage 2 门禁，不影响 Stage 1 退出）

1. 联动组 G1/G2/G4/G5 + 200×200 压测（Stage 2A 前置，债务 #23）。
2. ADR-NEW-14 阈值回填、R2/R3 门禁耗时实测（R2 门禁启用时）。
3. 全序键定稿（Stage 2B 冲突模型）、并行快照+串行提交（Stage 2B）。
4. ADR-NEW-19 模块清单全量定稿、滚动摘要实测（模拟核心完备后）。
5. R4 跨平台黄金复验、TestU01/PractRand 统计测试、正式性能基准（发布前）。
