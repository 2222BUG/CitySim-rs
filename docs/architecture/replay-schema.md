# 回放输入 / 模拟配置 Schema（Stage 3 T2，债务 #10）

- 状态：Proposed（2026-11-22；实装随本 schema 一并落盘于 crates/sim-replay）
- 关联：2.7（机器性能调整"写入存档和回放输入"）、2.9 条 8（玩家命令带生效 tick 并
  记录供回放）、ADR-NEW-14 条 6（LOD 切换动作记录）、EC-03（Schema 版本化/迁移
  可诊断口径）、ADR-NEW-19（canonical 编码规则共用）

## 1. 定位

回放输入是**权威运行的输入侧完整记录**：种子+模拟配置+全部输入事件。回放等价=
同输入两次运行 digest 相等。与存档（EC-03）分工：存档记录**状态**（含在途任务），
回放输入记录**输入**；存档可在元数据中引用 ReplayInput digest 以绑定回放身份。

## 2. Schema（version = 1）

```
ReplayInput {
    schema_version: u16,        // =1；变更语义须 bump（EC-03 bump 规则同口径）
    seed: u64,                  // 全局种子（ADR-NEW-22 口径）
    tick_rate_num: u64,         // tick 频率分子（dt=den/num 派生，2.4 from_ratio）
    tick_rate_den: u64,
    events: [InputEvent],       // 记录序=发生序；回放按 tick 提交（见 §4）
}

InputEvent =                       // 判别 tag u8
  | Command { command_id: u64, effective_tick: u64, agent_id: u64,
              opcode: u32, arg0: i64, arg1: i64 }        // 2.9 条 8（对应 sim-core PlayerCommand）
  | ConfigChange { tick: u64, key: u64, value: i64 }      // 2.7 tick 边界配置变更（key=稳定键哈希）
  | LodSwitch { tick: u64, entity_id: u64, from_level: u8, to_level: u8 }  // ADR-NEW-14 条 6
  | BudgetAdjustment { tick: u64, system_id: u64, per_tick_cap: u64 }      // 2.7 预算调整
```

- 编码：canonical 小端（sim-codec，module="sim-replay" kind="ReplayInput"），
  与存档共用底层编码规则、顶层 Schema 独立（ADR-NEW-19 决策）；
- 尾字节拒绝、版本不符可诊断（RPL-0001）——EC-03 同口径。

## 3. 回放等价（验收判据）

录制运行（Recorder 收集 events）→ encode → decode → 回放运行（Replayer 按 tick
提交事件）→ 两运行 digest 相等。digest 覆盖：状态 + 输入事件序列。

## 4. 提交语义

- Command 按 `effective_tick` 生效（2.9 条 8；录制时即带生效 tick）；
- ConfigChange / BudgetAdjustment / LodSwitch 仅允许 tick 边界（录制 API 断言
  tick 单调非降（Command 事件豁免：按提交序记录、生效 tick 可倒序，全序由
  (effective_tick, command_id) 在回放期重排保证——2026-11-23 I3 集成修订）；回放在各 tick 的事件按记录序提交=确定性）；
- 回放不重算决策：事件即事实（寻路/预算让步等系统级派生行为由确定性模拟自身重放）。

## 5. 版本 bump 规则

新增可选事件类型=可 bump 可不 bump（解码端拒绝未知 tag 时必须 bump 并提供迁移）；
字段语义/顺序/编码变更必须 bump；bunch 规则与 EC-03 Schema 版本一致（迁移失败可诊断）。
