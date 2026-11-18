# 错误码规范（EC-08 配套文档片段，Stage 1 T8）

- 日期：2026-11-19
- 依据：主文档 4.4 EC-08（Fatal/Recoverable/Diagnostic + SimErrorContext）、
  2.9 确定性边界（u64 tick、无时钟）
- 载体：`crates/sim-error`（ErrorCode / SimError / DiagnosticSink）

## 格式

```
MOD-NNNN
```

- `MOD`：3 位大写模块前缀（见登记表）。
- `NNNN`：4 位十进制编号，模块内从 0001 起；**编号一经发布不得复用**
  （废弃码保留占位并标 Deprecated）。

## 分层语义（EC-08）

| 层 | 含义 | 处置 |
| --- | --- | --- |
| Fatal | 不可恢复：存档损坏、协议版本不兼容、不变量破坏 | 终止本次运行/拒绝加载 |
| Recoverable | 可恢复：有确定性处置后模拟继续 | 记录 + 确定性处置（如重入队、兜底） |
| Diagnostic | 诊断信息，不改变权威状态 | 入诊断通道（确定性 Vec，可入 GlobalDigest） |

## SimErrorContext

| 字段 | 类型 | 语义 |
| --- | --- | --- |
| tick | u64 | 权威时间（2.9 条 7，不读系统时钟） |
| entity_id | u64 | **= AgentId**（2.3：与 Bevy Entity 解耦的稳定身份；正式类型随 T4） |
| system | &'static str | 产生点系统标识（namespace 口径） |

## 模块前缀登记

| 前缀 | 模块 | 已登记码 |
| --- | --- | --- |
| SAV | 存档/读档（sim-save） | SAV-0001 存档损坏、SAV-0002 Schema 不兼容 |
| PTH | 寻路（sim-pathfinding） | PTH-0001 过期结果重入队、PTH-0002 重算超限受阻兜底 |
| BGT | 预算与降级（sim-core） | BGT-0001 预算压力提示 |

## 新增流程

1. 新模块前缀与首批错误码在本表登记（随实现提交）。
2. 分层归属（Fatal/Recoverable/Diagnostic）与处置语义写入对应 ADR 或任务文档。
3. 逐条进条件核销清单/decision-log 备查；编号不得跨模块复用。
