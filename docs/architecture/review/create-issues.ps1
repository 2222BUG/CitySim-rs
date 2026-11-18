# CitySim-rs 评审 issue 批量创建（需先安装 gh CLI 并 gh auth login）
# 用法：pwsh docs/architecture/review/create-issues.ps1
$repo = "2222BUG/CitySim-rs"
$template = "`n## 裁决记录`n- 裁决：采纳 / 有条件采纳 / 拒绝 / 延后`n- 理由：`n- 验证条件："

$issues = @(
    @{ t = "[评审邀请] CitySim-rs V1.9.3 公开评审——ADR-NEW-14~23 裁决"; b = "CitySim-rs 技术方案 V1.9.3 处于公开评审阶段。Stage 0 工程验证已完成（9 步、51 测试、8 份实测报告），现邀请对 10 份 Proposed ADR 正式裁决。`n`n- 评审问题（12 问）与裁决模板：docs/architecture/review/REVIEW-INVITE.md`n- ADR：docs/architecture/adr/（ADR-NEW-14 至 ADR-NEW-23）`n- 实测证据：docs/benchmarks/`n- 裁决口径：采纳 / 有条件采纳 / 拒绝 / 延后，记录理由与验证条件`n`n请在子 issue 下按模板回复。" },
    @{ t = "[ADR-NEW-14] 预算与降级必须确定性——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-14-预算与降级必须确定性.md`n关联：2.7 / EC-04 / EC-11 / Stage 2C；评审问题 #4 #10`n决策要点：预算/降级只用确定性工作量单位；墙钟仅遥测；tick 边界显式变更入存档回放；LOD 切换同为确定性信号。$template" },
    @{ t = "[ADR-NEW-15] 确定性边界定义——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-15-确定性边界定义.md`n关联：2.9 / EC-05 / EC-12；评审问题 #7`n决策要点：2.9 八条契约；clippy 分层已实测可行（docs/benchmarks/clippy-layering-REPORT.md）。$template" },
    @{ t = "[ADR-NEW-16] 存档最小集合扩充——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-16-存档最小集合扩充.md`n关联：EC-03 / EC-11 / 2.3；评审问题 #6`n决策要点：七类字段最小集；在途任务定案保存并恢复。实证：save-restore-spike 9/9 含强读档等价。$template" },
    @{ t = "[ADR-NEW-17] 过期寻路结果策略——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-17-路网事务的过期结果策略.md`n关联：EC-01 / 2.5 / 2.6；评审问题 #3 #4`n决策要点：影响区域相交判断（含聚合规则）；只保证合法性；前向进展不变量+重算上限+兜底候选。实证：network-tx-spike 8/8。$template" },
    @{ t = "[ADR-NEW-18] 参数基线与数值契约补充——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-18-仿真参数基线与数值契约补充.md`n关联：2.10 / 2.4 / Spike A；评审问题 #1`n定案值（2026-11-18 项目负责人）：30 Hz / 16 km×16 km / 1 mm / 50 m·s⁻¹ / 10¹² 独立 i64 / 100k Agent×1k Tick / 10⁹ ticks。$template" },
    @{ t = "[ADR-NEW-19] GlobalDigest 与规范化序列化——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-19-GlobalDigest摘要与规范化序列化.md`n关联：EC-05 / EC-03 / EC-11；评审问题 #6`n决策要点：BLAKE3-256；canonical 小端编码（与存档共用编码器）；digest_version u16；存档点全量重算。$template" },
    @{ t = "[ADR-NEW-20] 请求/事务 ID 与在途寻路口径——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-20-请求事务ID分配与在途寻路任务口径.md`n关联：2.3 / EC-03 / 2.8 / ADR-NEW-16`n决策要点：请求/事务 ID 单调分配器高水位入存档；在途寻路=单 tick 原子搜索（强读档等价天然成立）。$template" },
    @{ t = "[ADR-NEW-21] SimScalar=Q32.32 定案——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-21-SimScalar格式定案.md`n关联：2.4 / Spike A；评审问题 #1`n决策要点：Q32.32(i64)、ties away、i128 中间值、u64 tick、dt from_ratio 黄金值 143,165,577。实证：Rust 12 测试、9 项硬约束 PASS（docs/benchmarks/Spike-A-SimScalar-REPORT.md）。$template" },
    @{ t = "[ADR-NEW-22] 系统级 RNG 协议——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-22-系统级RNG协议.md`n关联：EC-02 / EC-03；评审问题 #2`n决策要点：双模式（纯派生零状态 + Stream64 16 B 存档）；SplitMix64 协议 v1；RNG_PROTOCOL_VERSION 入存档。实证：rng-spike 9/9。$template" },
    @{ t = "[ADR-NEW-23] 过期结果细则（相交粒度/重算上限/兜底/行走删路）——裁决"; b = "文件：docs/architecture/adr/ADR-NEW-23-路网事务过期结果细则.md`n关联：EC-01 / ADR-NEW-17 / ADR-NEW-20；评审问题 #3 #4`n决策要点：包围盒 1 mm 网格并集聚合；recompute_limit=3；兜底默认 MarkBlocked；行走删路 RerouteNearest 优先。实证：network-tx-spike 8/8（docs/benchmarks/Spike-C-NetworkTx-REPORT.md）。$template" }
)

foreach ($i in $issues) {
    gh issue create --repo $repo --title $i.t --body $i.b
}
Write-Host "done: $($issues.Count) issues"
