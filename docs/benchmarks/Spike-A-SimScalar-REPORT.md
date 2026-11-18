# Spike A: SimScalar — preliminary report (Status: Proposed)

> 入仓版（2026-11-18）。由评审阶段收到的 REPORT.md 修订而来（P3/P6 修正、Rust 实测补记）。
> 证据纪律：全文状态 Proposed；下文 VERIFIED 仅指本文档记录的实验，不代表 SimScalar 格式已定案（2.10 参数基线未定，格式选择依赖它）。

## What is verified vs. not
- VERIFIED (原稿作者会话): `analysis.py`，Q48.16 / Q32.32 / Q24.40 / Q16.48 的 i64 精确整数模拟。运行输出：同目录 `analysis-output.txt`。
- VERIFIED (2026-11-18，rustc 1.100.0): `spikes/simscalar-spike` 编译通过（零警告），`cargo test` **7/7 通过**（含黄金漂移测试、FNV 摘要常量、10 万次属性测试）。原稿"lib.rs 未编译验证"就此升级。
- NOT VERIFIED: 跨平台运行（x86_64 vs aarch64）、真实地图/速度/经济值域需求、i128 通路性能基准。
- NOT DONE: 参数基线（2.10）定案、SimScalar 格式定案（依赖参数基线）。

注（P7）：主文档 2.1 版本基线写 Rust 1.89（"待复现验证"），本次实测工具链为 stable **1.100.0**。收敛方式（rust-toolchain.toml 锁定或更新基线并记录）见 decision-log。

## Findings from analysis.py (assumptions: 15 m/s, 1e6 ticks, as stated in the script)

| Format | Range | ulp | Drift @20TPS, round | Verdict |
|---|---|---|---|---|
| Q48.16 | ±1.4e14 | 1.5e-5 | +45.8 m | Reject: resolution too coarse |
| Q32.32 | ±2.1e9 | 2.3e-10 | +0.7 mm | Leading candidate |
| Q24.40 | ±8.4e6 | 9.1e-13 | +0.003 mm | Viable if all coordinates and values stay under 8.4e6 |
| Q16.48 | ±3.3e4 | 3.6e-15 | ~0 | Reject: 地图宽度上限 65.5 km（±32,768 m）——64 km 地图（±32,000 m）恰好容得下，>65.5 km 排除（P3：原稿"排除 64 km+ 地图"表述不准确） |

1. Time must not be SimScalar seconds: Q32.32 overflows after ~68 years，**Q24.40 约 0.3 年即溢出（P6：原稿漏列）**，Q16.48 immediately. Use a u64 tick counter.
2. Truncation (floor) biases results one way (-2.8 mm vs +0.7 mm with round-to-nearest on Q32.32), so rounding mode must be fixed by contract.
3. 1/20 and 1/60 are not exactly representable; always derive dt with `from_ratio` and keep the rounding error in the golden tests.（2.10 候选 10/20/30 Hz 同样非精确；见同目录 `tick-rate-drift.py`，含 16/32 Hz 参考行：dt 精确可表示、漂移为 0）
4. All i64 formats need i128 intermediates for mul/div; the cost must be benchmarked in Rust.

## Open inputs needed from the project owner
Max map extent, minimum spatial resolution, max speed, tick rate，**最大持续运行 tick 数（最长连续运行时长，决定漂移预算口径）**, and the largest economy value. The choice between Q32.32 and Q24.40 depends on these.

## Next
1. ~~cargo test the crate~~ DONE（7/7，rustc 1.100.0）。
2. Benchmark mul/div (i128 path) on x86_64 and aarch64.
3. Write the ADR once the requirement numbers are fixed.

## 修订记录（2026-11-18 评审修订）
- P1：crate 以 `spikes/simscalar-spike/`（src/ 布局、`license` 字段、`#![forbid(unsafe_code)]`）入仓。
- P2：`analysis.py` 以"复核重建版"入仓 `docs/benchmarks/`，运行输出一并入仓；重跑结论与 V1.9.3 引用一致（68 年溢出 ✓ 截断单向偏差 ✓）。
- P3：Q16.48 拒绝理由改为"地图宽度上限 65.5 km"。
- P4/P5：新增 `tick-rate-drift.py`（10/20/30 Hz 决策辅助 + 最大持续运行 tick 数漂移预算），输出入仓。
- P6：findings 1 补 Q24.40 秒累积 0.3 年溢出。
- P7：补记 Rust 实测结果与工具链版本差异。
