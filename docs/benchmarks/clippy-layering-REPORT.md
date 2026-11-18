# clippy 分层（EC-12）可行性实测报告

- 日期：2026-11-18
- 状态：验证记录（对应主文档 4.7 EC-12 契约即代码、2.9 条 3/4/5、ADR-NEW-15 验证方法；
  decision-log 待决事项"clippy disallowed 自动检查可行性"）
- 环境：rustc/clippy 1.100.0 (stable-x86_64-pc-windows-msvc)
- 探针：`_clippy_probe/`（临时 crate，实测后不入仓；配置与结论见下）

## 实测配置（权威侧口径）

```toml
# clippy.toml（放在权威 crate 包根，逐 crate 生效——即分层机制本身）
disallowed-types = ["std::collections::HashMap", "std::collections::HashSet", "f32", "f64"]
disallowed-methods = ["std::time::Instant::now", "std::time::SystemTime::now"]
```

## 疑点与结论

| # | 疑点（源自 eng-audit 覆盖缺口清单） | 实测结果 | 结论 |
| --- | --- | --- | --- |
| 1 | HashMap/HashSet 类型使用能否被抓 | use 语句 + 类型标注 + 构造共 3 处全部报告（clippy::disallowed_types，配置后默认 warn） | ✅ 有效 |
| 2 | Instant::now / SystemTime::now 能否被抓 | 两处均报告（clippy::disallowed_methods） | ✅ 有效 |
| 3 | `f32` 平台路径 vs `std::primitive::f32` 全路径写法是否漏报 | **两种写法均被抓**（含 `1.0 as std::primitive::f32`） | ✅ 路径写法疑点解除 |
| 4 | 分层豁免机制（渲染/UI/遥测 crate 不受限） | 函数级 `#[allow(clippy::disallowed_types)]`/`#[allow(clippy::disallowed_methods)]` 零报告；crate 级同理；clippy.toml 按包根查找=天然按 crate 分层 | ✅ 有效 |
| 5 | 第三方 crate 内部使用 | clippy 只分析本包源码，依赖 crate 内部不报告 | ⚠️ 覆盖缺口仍在（护栏定位不变：本包代码 + 依赖评审） |
| 6 | 宏展开后的用法 | 未单独构造用例；lint 在 HIR 层报告，宏展开后大概率可见 | ⚠️ 未验证，正式权威 crate 建库时补一个宏用例 |

## 落地验证（五 crate 终态）

| crate | 侧 | clippy.toml | 结果 |
|---|---|---|---|
| spikes/simscalar-spike | 权威侧 | 严格 | CLEAN（含 --all-targets） |
| spikes/rng-spike | 权威侧 | 严格 | CLEAN（bench 模块遥测豁免） |
| spikes/network-tx-spike | 权威侧 | 严格 | CLEAN（perf 测试遥测豁免） |
| spikes/save-restore-spike | 权威侧 | 严格 | CLEAN（Stream64::next 命名豁免 + is_multiple_of 整改） |
| crates/sim-test-support | **非权威侧（测试基建）** | **不放** | 无 disallowed 报告（不受限） |

回归：五 crate 49 测试全部通过（11+12+9+8+9），黄金值未变动。

## 豁免登记（遥测侧口径，2.9：浮点/墙钟仅渲染/UI/遥测）

- rng-spike `mod bench`、simscalar-spike `perf_smoke_mul`、network-tx-spike `perf_smoke`：
  `#[allow(clippy::disallowed_types, clippy::disallowed_methods)]`——墙钟与 f64 仅用于
  测量输出（ns/op 换算），不进入权威模拟结果。
- save-restore-spike `Stream64::next`：`#[allow(clippy::should_implement_trait)]`——
  语义为"流的下一值"而非迭代器协议。

## 结论

clippy disallowed-types/methods 自动检查**可行**，按 crate 分层（clippy.toml 放置与否）+
函数级 allow 豁免的组合可完全落实 ADR-NEW-15 条 3/4/5 的落地手段。护栏定位不变：
这是语法面护栏（第三方内部与宏展开仍有缺口），语义级"迭代顺序依赖"仍靠
代码评审 + property 测试兜底。CI 中以 `-D warnings` 提升为合并门禁（见
`.github/workflows/ci.yml` ec12-lint job）。

## 复现

```
cargo clippy --all-targets          # 在各 crate 目录下
```
