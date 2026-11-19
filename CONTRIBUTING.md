# 参与贡献指南（CONTRIBUTING）

感谢关注 CitySim-rs！本项目处于**引擎地基与文档驱动开发**阶段——此时参与设计，比等游戏做出来再参与更有影响力。

> 基本礼仪：善意推定、对事不对人；不同意任何决策都欢迎，但请给出理由（这正是本项目决策流程的核心）。

## 参与路径（按门槛从低到高）

| 路径 | 做什么 | 入口 |
| --- | --- | --- |
| 🟢 小任务 | 认领 [good first issue](https://github.com/2222BUG/CitySim-rs/labels/good%20first%20issue) | 直接在 issue 下留言认领 |
| 💬 设计意见 | 对 ADR / 未定案决策发表意见（同意、反对、替代方案皆可） | [Issues](https://github.com/2222BUG/CitySim-rs/issues)、[Discussions](https://github.com/2222BUG/CitySim-rs/discussions) |
| 📄 文档改进 | 主文档/ADR 挑错、英文化翻译 | PR 或 issue |
| 🔧 代码贡献 | 修 bug、补测试、实现任务清单条目 | PR |
| 📐 架构设计 | 认领 [decision-log](docs/architecture/decisions/decision-log.md) 中的决策债务 | 在对应 issue/PR 中认领 |

## 开发环境

- Rust **1.100.0**（`rust-toolchain.toml` 自动锁定；`clippy` 组件已列入）
- 无需任何外部依赖；Windows / Linux / macOS 均可（CI 跑 Linux）

```bash
cargo test --workspace        # 全量测试（124 个，应当全绿）
cargo clippy --workspace --all-targets -- -D warnings
python3 .github/scripts/dep_graph_check.py   # 依赖方向检查（CI 强制）
```

## 代码规范（本仓库特有的硬约束）

这些不是风格偏好，是**确定性契约**（主文档 2.9）的落地，CI 会拦：

1. **权威 crate**（见 [crate-authority-list.md](docs/architecture/crate-authority-list.md)）：
   - 禁 `f32`/`f64`（权威状态用 SimScalar 定点数）；
   - 禁 `HashMap`/`HashSet` 迭代顺序依赖（用 `BTreeMap` 或显式排序）；
   - 禁 `Instant::now`/`SystemTime::now`（权威时间只有 u64 tick）；
   - 以上由各 crate 根 `clippy.toml` 强制（`cargo clippy -D warnings` 即门禁）。
2. **依赖方向**：模拟核心零依赖；权威层不得依赖非权威层；`sim-test-support` 只能出现在 dev-dependencies。违反会被 `dep_graph_check` 拦下。
3. 新增 crate 须先在 `crate-authority-list.md` 登记权威/非权威分类。
4. 测试黄金值**必须先运行打印、再固化**，禁止凭计算或想象编造常量（证据纪律）。

## 文档与决策规范

- **状态词表**（主文档 0.2）：Proposed / Accepted / Implemented / Verified / Deprecated，不得把设计目标写成既成事实。
- 涉及决策的变更：在 [decision-log](docs/architecture/decisions/decision-log.md) 记录裁决与理由；条件未清不得开工对应模块（0.2 注记规范）。
- 实验性结论必须可复现：脚本与输出入仓 `docs/benchmarks/`。
- 主文档（MD）与评审流转副本（docx）需保持同步；改动主文档时留意 0.1 映射表的可追溯性。

## PR 要求

1. 一个 PR 一件事；标题沿用现有风格（`feat(stage2a): ...` / `docs(...)` / `fix(...)`）。
2. 测试全绿 + clippy CLEAN + dep_graph_check 通过（本地跑一遍）。
3. 权威逻辑的变更附测试；行为契约变化更新对应 ADR/decision-log 行。
4. 跑不动测试的纯文档 PR 也欢迎——注明即可。

## 认领与协作

- issue 下留言"认领"即锁定；两周无进展可由他人接手（无压力，欢迎放鸽子前说一声）。
- 大改动先开 issue 讨论方案再动手（省返工）。
- 有疑问直接问——issue 里没有蠢问题。

## 行为准则

尊重、包容、就事论事。恶意行为（骚扰、歧视、人身攻击）零容忍，维护者有权移除参与资格。
