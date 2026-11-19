# CitySim-rs

**基于 Rust + Bevy 的开源城市模拟引擎——确定性优先，文档驱动开发。**

[![CI](https://github.com/2222BUG/CitySim-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/2222BUG/CitySim-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#许可证)
[![Rust](https://img.shields.io/badge/Rust-1.100.0-orange.svg)](rust-toolchain.toml)
![Tests](https://img.shields.io/badge/tests-124%20passed-brightgreen.svg)

> [English](README.en.md) | 简体中文

**为什么与众不同**：同种子两次运行摘要一致、存档-读档后继续模拟与直接模拟**逐位相等**（强读档等价）、权威状态零浮点零时钟——这些不是愿景，是**挂着测试的承诺**。整个设计过程由 ADR + 决策债务台账 + 证据纪律驱动，历经六家独立 AI 评审共识裁决，全部可追溯。

## 项目状态

| 层 | 状态 |
| --- | --- |
| 设计 | [技术方案 V1.9.3](docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md)（唯一现行主文档）；10 份 ADR（NEW-14~23）已正式裁决（Accepted，附条件）；[26 项决策债务](docs/architecture/decisions/decision-log.md)台账管理 |
| Stage 0 | ✅ 关闭（2026-11-19，[退出核对 6/6](docs/benchmarks/Stage-0-Exit-CHECKLIST.md)）：参数基线定案、5 个 Spike 验证、依赖五件套、CI 骨架 |
| Stage 1 | ✅ 完成（2026-11-19，[退出核对 6/6](docs/benchmarks/Stage-1-Exit-CHECKLIST.md)）：分层 crate 骨架、SimScalar、RNG 协议、GlobalDigest、存档/读档、错误分层 |
| Stage 2 | ✅ 完成（2A 路网/A*、2B 并行确定性、2C 交通与降级；[2A](docs/benchmarks/Stage-2A-Exit-CHECKLIST.md)/[2B](docs/benchmarks/Stage-2B-Exit-CHECKLIST.md)/[2C](docs/benchmarks/Stage-2C-Exit-CHECKLIST.md) 退出核对） |
| 游戏本体 | 尚无——当前是引擎地基与可验证文档；欢迎一起设计 |

## 快速开始

```bash
git clone https://github.com/2222BUG/CitySim-rs.git
cd CitySim-rs
cargo test --workspace        # 124 测试全绿（Rust 1.100.0，rust-toolchain.toml 已锁定）
cargo test -p sim-save        # 强读档等价：存档读档跑至 M == 直接跑至 M
```

无任何外部依赖要求；`cargo +1.89 check`（Bevy 0.18.1 MSRV 参考线）亦须通过（CI 门禁）。

## 你可以参与什么

- 🟢 **[good first issue](https://github.com/2222BUG/CitySim-rs/labels/good%20first%20issue)**——从小任务开始：统计测试、基准、翻译、设计 RFC
- 💬 **ADR 评审与设计讨论**——[Issues](https://github.com/2222BUG/CitySim-rs/issues) 与 [Discussions](https://github.com/2222BUG/CitySim-rs/discussions)：很多决策**尚未定案**，提出意见即贡献
- 📐 **架构设计**——26 项决策债务（Agent 状态机、回放 schema、LOD……）都开放认领，见 [decision-log](docs/architecture/decisions/decision-log.md)
- 详见 [CONTRIBUTING.md](CONTRIBUTING.md)

## 文档导航

| 想了解 | 去哪读 |
| --- | --- |
| 整体设计 | [技术方案 V1.9.3](docs/architecture/CitySim-rs-Technical-Spec-V1.9.3.md) |
| 决策如何产生、哪些还悬着 | [decision-log](docs/architecture/decisions/decision-log.md)（裁决记录 + 26 项债务 + 条件核销清单） |
| 单项架构决策 | [`docs/architecture/adr/`](docs/architecture/adr/)（ADR-NEW-14 ~ 23，含正式裁决节） |
| 确定性承诺的证据 | [determinism-evidence.md](docs/architecture/determinism-evidence.md)、[benchmarks/](docs/benchmarks/)（黄金向量、实验脚本、退出核对） |
| 参与评审 | [AI-REVIEW-BRIEF](docs/architecture/review/AI-REVIEW-BRIEF.md)、[REVIEW-INVITE](docs/architecture/review/REVIEW-INVITE.md) |

## 仓库结构

```
crates/                     # 正式 crate（2.2 分层；权威/非权威见 crate-authority-list.md）
├─ sim-core / sim-agent / sim-network / sim-pathfinding / sim-traffic / sim-economy   # 权威模拟层
├─ simscalar / sim-rng / sim-codec / sim-digest / sim-error / sim-save               # 权威基础层
└─ sim-view / sim-test-support / dep-verify                                          # 非权威工具层
spikes/                     # Spike 验证件（Stage 0 证据，保留可复现）
docs/
├─ architecture/            # 主文档、ADR、decision-log、任务清单、评审资料
└─ benchmarks/              # 可复现实验脚本、黄金向量、基准与退出核对
```

依赖方向由 CI 强制（`dep_graph_check`）：模拟核心不依赖 UI、权威层不依赖工具层、验证件不进入正式依赖。

## 许可证

MIT OR Apache-2.0 双许可，任选其一。详见 [LICENSE-MIT](LICENSE-MIT) 与 [LICENSE-APACHE](LICENSE-APACHE)。
