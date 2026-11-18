# Stage 0 退出条件核对（Step 9）

- 日期：2026-11-18
- 依据：主文档第 5 章 Stage 0（退出条件 6 条，MD L932-942）
- 结论：**6/6 满足（2026-11-19 正式裁决完成后收官）——Stage 0 关闭**

| # | 退出条件（原文） | 状态 | 证据 |
| --- | --- | --- | --- |
| 1 | 高风险设计假设有实验数据 | ✅ | Spike A（SimScalar：范围/漂移/位宽/舍入，12+9 测试，Spike-A-SimScalar-REPORT）、Spike B（RNG 隔离/调度无关/协议版本化，9 测试，Spike-B-RNG-REPORT）、Spike C（路网事务原子发布/过期结果/前向进展，8 测试，Spike-C-NetworkTx-REPORT）、存档兼容（9 测试，SaveRestore-Validation-REPORT）；全部实测数据入仓 docs/benchmarks/ |
| 2 | 关键数据语义有明确决策 | ✅ | 决策内容明确且已成文：SimScalar=Q32.32（ADR-NEW-21）、RNG 协议双模式+SplitMix64（ADR-NEW-22）、过期结果细则（ADR-NEW-23）、GlobalDigest 方案（ADR-NEW-19）、请求/事务 ID 与在途任务口径（ADR-NEW-20）、存档"保存并恢复"（ADR-NEW-16 修订）。2026-11-19 正式裁决完成：全部转 Accepted（附条件），裁决记录见 decision-log |
| 3 | 参数基线表（2.10）已由项目负责人确定并记录 | ✅ | Step 1 定案（CS2 参考值七项），decision-log 记录 |
| 4 | 核心工程能够稳定编译 | ✅ | workspace 建立（根 Cargo.toml，6 成员）；`cargo test --workspace` **51/51 全过**；`cargo build` exit 0；`cargo +1.89 check --workspace --all-targets` 全闭包 exit 0（ordered-float 锁 5.4.0） |
| 5 | 基础测试框架可运行 | ✅ | crates/sim-test-support（11 测试：fixtures/deterministic 含读档等价模板/property/benchmark/assertions/chaos）；CI R0/R1 门禁（.github/workflows/ci.yml） |
| 6 | 关键 ADR 已完成评审 | ✅ | 2026-11-19 五家独立 AI 评审共识裁决完成：NEW-21 采纳、NEW-14/15/16/18/19/22 有条件采纳、NEW-17/20/23 联动组定案实施前置（G1-G5）；全部转 Accepted（附条件），见 decision-log"2026-11-19 正式裁决记录" |

## 处置（2026-11-19 收官）

- 条件 2/6 已随 2026-11-19 正式裁决一并消化：10 项 ADR 裁决记入 decision-log 与各 ADR"正式裁决"节。
- **Stage 0 关闭**；遗留债务（#11 编码器、#20 预算滚动、#23 前向进展 G1-G5、#24 ID 分配点、#25 namespace 校验、#26 CSR+A* 确定性细则、#22 工具链收敛、#21 ruzstd 安全债）转 Stage 1/2A 开工门禁。
- 待项目负责人拍板：状态口径（"Accepted（附条件）" vs 严格口径"条件满足前保持 Proposed"）与 Rust 基线 1.89 vs 1.100.0（债务 #22）。

## workspace 结构（Step 9 建立）

```
Cargo.toml            # workspace：resolver=2，6 成员
Cargo.lock            # 唯一锁文件（ordered-float 钉 5.4.0 保 Rust 1.89 闭包）
spikes/               # simscalar-spike / rng-spike / network-tx-spike / save-restore-spike
crates/               # sim-test-support / dep-verify
```

成员 Cargo.lock 已移除（workspace 统一锁文件；ordered-float ≤5.4.0 锁定条件随之落在根 Cargo.lock）。
权威/非权威 crate 划分待 2.2 定案（decision-log 债务 #15）。

## 复现

```
cargo test --workspace                  # 51 tests
cargo build                             # 目标平台构建
cargo +1.89 check --workspace --all-targets   # MSRV 闭包
cargo audit                             # 安全审计（ruzstd 债见 #21）
```
