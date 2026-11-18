# 待拍板决策请求（2026-11-19）

背景：CitySim-rs 技术方案 V1.9.3 的 10 个 ADR 已完成六家独立 AI 评审共识裁决
（NEW-21 采纳；NEW-14/15/16/18/19/22 有条件采纳；NEW-17/20/23 联动组定案实施前置），
Stage 0 已关闭。现有 **2 件事项需项目负责人/评审方拍板**，欢迎各界（人类评审与 AI）给出意见。

回传方式：在 GitHub issue 对应条目下评论（推荐），或把回执交给转发人。
回传格式（每个决策一行）：

```
决策 1：A / B
决策 2：A / B
（理由可选，一句话）
```

---

## 决策 1：ADR 状态口径（附条件裁决如何记状态）

**背景**：10 个 ADR 的裁决里有 9 个是"有条件采纳"（验证条件作为后续开工门禁），
但主文档 0.2 的状态词表（Proposed / Accepted / Implemented / Verified / Deprecated）
没有"附条件"中间态。两种记法：

| 选项 | 内容 | 主张方 | 代价 |
| --- | --- | --- | --- |
| **A（维护者推荐）** | 记 **Accepted（附条件）**：裁决=接受，条件作为 Stage 1/2A 开工门禁在 decision-log 追踪 | 维护者（0.2 词表无中间态；Stage 0 退出条件 6 是"完成评审"而非"全部无条件 Accepted"） | 状态列带括号注记，需人工维护条件清单 |
| **B（严格口径）** | 条件满足前保持 **Proposed**，条件逐项满足后一次性转 Accepted | 评审 F | 语义最严谨；但 Stage 0 退出条件 6 需另行解释（评审本身已完成） |

**影响面**：10 个 ADR 的状态列、decision-log 裁决表、Stage-0-Exit-CHECKLIST 的表述。
两选项都不改变任何技术决策内容。

---

## 决策 2：Rust 工具链基线（债务 #22）

**背景**：文档 2.1 写的基线是 Rust 1.89（= Bevy 0.18.1 官方 MSRV，Step 8 实测吻合），
但 workspace 实测用的是 Rust 1.100.0（Step 8/9 的 51 测试、CI 主 job 均在其上）。
两者都已验证可用：1.89 闭包成立（需 Cargo.lock 钉 ordered-float 5.4.0，债务 #22 线索）。

| 选项 | 内容 | 代价 |
| --- | --- | --- |
| **A** | 锁 **1.89**（Bevy MSRV，与文档 2.1 一致） | 新语言特性受限；ordered-float 钉 5.4.0 成为长期约束 |
| **B** | 升 **1.100.0**（实测工具链，CI 主 job 现用） | 文档 2.1 基线句改写；MSRV 语义需三区分（文档基线 / CI 最低 / 开发工具链），CI 保留 1.89 check job 或删除 |

**影响面**：rust-toolchain.toml、CI 矩阵（msrv-check job）、2.1 文档、债务 #22 关闭方式。

---

裁决全档：[decision-log](../decisions/decision-log.md)"2026-11-19 正式裁决记录"节 ·
评审归档：[archive/](archive/) · 评审任务书：[AI-REVIEW-BRIEF.md](../AI-REVIEW-BRIEF.md)
