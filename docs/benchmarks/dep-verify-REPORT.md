# Step 8 依赖与工具链验证报告（2.1"正式锁定前必须完成"五件套）

- 日期：2026-11-18
- 状态：验证记录（对应主文档 2.1 技术基础；Stage 0 Step 8）
- 工程：`crates/dep-verify/`（bevy="=0.18.1"、bevy_egui="0.39"、rapier2d/rapier3d="0.33"）
- 环境：Windows x86_64（x86_64-pc-windows-msvc）；rustc 1.89.0 (29483883e 2025-08-04) 与 rustc 1.100.0 (0ef1bc386 2026-11-09) 双工具链；cargo-audit 0.23.0

## 五件套结果

| # | 检查项（2.1 原文） | 结果 | 证据 |
| --- | --- | --- | --- |
| 1 | 依赖版本兼容检查 | ✅ 零冲突解析 | `cargo generate-lockfile`：bevy 0.18.1 / bevy_ecs 0.18.1 / bevy_egui 0.39.1 / egui 0.33.3 / rapier2d 0.33.0 / rapier3d 0.33.0 共存 |
| 2 | 最小工程编译 | ✅（附条件，见发现 1） | `cargo +1.89 check --all-targets` exit 0（全依赖闭包在 Rust 1.89 编译通过） |
| 3 | 目标平台构建 | ✅ | `cargo build`（x86_64-pc-windows-msvc，dev profile）exit 0 |
| 4 | 关键 API 验证 | ✅ | `cargo test` 2/2：`key_apis_are_usable`（Bevy App+ResMut 系统调度两 update=2；bevy_egui EguiPlugin/EguiContexts 类型解析；rapier3d RigidBodyBuilder::dynamic()、rapier2d ColliderBuilder::ball(0.5) 构建器断言） |
| 5 | 依赖许可证及安全检查 | ⚠️ 许可证 ✅ / 安全 ⚠️ 见发现 2 | 许可证 9 项全兼容；cargo-audit：1 高危 + 2 unmaintained 警告 |

## 版本与 MSRV 实测

| crate | 锁定版本 | crates.io latest | MSRV（rust-version） |
| --- | --- | --- | --- |
| bevy | 0.18.1（=0.18.1 钉死） | 0.20.1 | 1.89.0 |
| bevy_egui | 0.39.1 | 0.43.1 | 1.89.0 |
| rapier2d / rapier3d | 0.33.0 | 0.36.1 | 1.86 |
| egui（传递） | 0.33.3 | - | - |
| ordered-float（传递） | **5.4.0（锁定）** | 5.5.x | 5.5.x 要求 1.90 |

## 发现 1（基线条件）：ordered-float 5.5.x 破坏 Rust 1.89 基线

- 现象：`cargo +1.89 check` 报 `ordered-float@5.5.1 requires rustc 1.90`；降级 5.5.0 仍报同错
  （该系列整体 MSRV=1.90）。
- 传递链：`parry2d/parry3d 0.28.0（rapier 侧）` 与 `wgpu-hal 27（bevy_render 侧）` 双链引入。
- 处置：`cargo update -p ordered-float --precise 5.4.0` 后 **1.89 全闭包 check 通过（exit 0）**。
- 结论：**"Rust 1.89"基线带锁文件条件成立**——必须以 Cargo.lock 锁定 ordered-float ≤5.4.0；
  不锁则闭包要求 rustc ≥1.90。基线表述与"1.89 vs workspace 实测 1.100.0"收敛取舍
  交项目负责人定案（decision-log 待决事项）。

## 发现 2（安全）：cargo-audit 全量审计（611 个依赖）

| advisory | crate | 级别 | 说明 | 处置 |
| --- | --- | --- | --- | --- |
| RUSTSEC-2026-0344 | ruzstd 0.8.3 | **高危 7.5** | 解压炸弹（小输入耗尽内存 OOM），修于 ≥0.9.1 | **基线内不可修复**：bevy_image 0.18.1 钉 ruzstd ^0.8，`cargo update -p ruzstd` 锁内无解（"Locking 0 packages"）。评估：位于纹理/资产解码路径（bevy_image，非权威侧，2.9 口径），Stage 0 不加载不可信资产；登记为安全债，随 Bevy 升级 ADR（2.1"Bevy 版本升级应通过 ADR"）消化 |
| RUSTSEC-2024-0436 | paste 1.0.15 | 警告（unmaintained） | 无修复版本 | 记录跟踪 |
| RUSTSEC-2026-0192 | ttf-parser 0.25.1 | 警告（unmaintained） | 无修复版本 | 记录跟踪（字体解析，UI 侧） |

## 许可证清点（cargo metadata）

| crate | license |
| --- | --- |
| bevy / bevy_ecs / egui | MIT OR Apache-2.0 |
| bevy_egui / ordered-float | MIT |
| rapier2d / rapier3d / parry2d / parry3d | Apache-2.0 |

全部与本项目 MIT OR Apache-2.0 双许可兼容；无 copyleft/专有许可证引入。

## 结论

1. 版本基线（Bevy 0.18.1 / bevy_egui 0.39.x / rapier 0.33.x）**验证通过**；"Rust 1.89"
   成立但带 ordered-float ≤5.4.0 锁定条件。
2. 关键 API 面（ECS 调度、UI 插件类型、2d/3d 物理构建器）可用；rapier 0.33 的
   `body_type()/shape()` 为方法访问（非公开字段）。
3. 安全面：1 高危（ruzstd，基线内不可修复）+2 unmaintained，均非权威模拟路径；
   建议在 Bevy 升级 ADR 中把 ruzstd 升级列为升级动机之一。
4. rust-toolchain.toml（仓库根）锁定 channel "1.100.0"（含具名工具链安装要求）；与 CI 一致。

## 复现

```
cd crates/dep-verify
cargo +1.89 check --all-targets   # MSRV 闭包编译（需 Cargo.lock）
cargo build                        # 目标平台构建
cargo test                         # 关键 API 验证 2/2
cargo audit                        # 安全审计
```
