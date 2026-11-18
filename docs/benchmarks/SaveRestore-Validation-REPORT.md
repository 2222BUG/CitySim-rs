# Step 5 存档兼容验证报告（EC-03 最小集 + 读档等价）

- 日期：2026-11-18
- 状态：验证记录（对应主文档 3.3 EC-03、EC-11 读档等价测试；Stage 0 Step 5）
- 原型：`spikes/save-restore-spike/`（零依赖、#![forbid(unsafe_code)]）
- 环境：rustc 1.100.0 (0ef1bc386 2026-11-09)、Windows x86_64（R4 跨平台未验）
- 结果：**9/9 测试全过、零警告**

## 验证项结论

| # | 验证项 | 对应条文 | 结果 | 证据测试 |
| --- | --- | --- | --- | --- |
| 1 | 最小兼容核心七类字段完备入档、完整回读 | EC-03 扩充表 | ✅ | `save_set_covers_ec03_categories` |
| 2 | 存档点仅 tick 边界 | EC-03 条文 | ✅ | `save_rejects_non_tick_boundary` |
| 3 | 强读档等价（N=40 存档读档跑至 M=100 == 直接跑至 100） | EC-11 R2 强形式 | ✅ | `strong_readback_equivalence` |
| 4 | 弱形式等价（两次独立存档-读档运行一致，若改"丢弃并重新入队"的降级口径） | EC-11 强/弱绑定 | ✅ | `weak_form_equivalence` |
| 5 | ID 高水位防复用（Agent/请求/事务三类） | 2.3、ADR-NEW-20 | ✅ | `id_high_water_prevents_reuse` |
| 6 | 有状态随机流回读后续列不断裂（16 B/流入档） | EC-02、ADR-NEW-22 | ✅ | `stateful_rng_roundtrip` |
| 7 | 篡改存档 → 一致性检查拒绝（摘要比对） | EC-03"读档后比对" | ✅ | `tampered_save_is_rejected` |
| 8 | 未知 Schema 版本可诊断失败 | EC-03"迁移失败可诊断" | ✅ | `unknown_schema_is_diagnosable` |
| 9 | 尾部多余字节拒绝（不静默丢弃） | EC-03"不兼容字段不能静默丢弃" | ✅ | `trailing_bytes_rejected` |

## 读档等价测试模板（可直接移入 EC-07 测试基建）

```
run_a: seed → 跑 N tick → 继续跑至 M → digest_A
run_b: seed → 跑 N tick → save() → load() → 跑至 M → digest_B
断言 digest_A == digest_B（强形式，依赖"保存并恢复"语义）
```

- 保存点上保留在途寻路请求队列与未发布路网事务（本原型在任意 tick 持续产生在途工作，
  保证保存点非空队列）。
- 弱形式（政策改为"丢弃并重新入队"时的降级口径）：两次独立 save/load 运行 digest 相等。

## 关键口径与限制

1. **摘要占位**：状态摘要为 FNV-1a 64 over canonical 正文（摘要槽 0 占位），是 GlobalDigest
   的**占位实现**；ADR-NEW-19（Proposed）裁决后替换为 BLAKE3-256 + digest_version，
   `state_digest()` 是唯一替换点。等价测试内部两侧同用占位算法，结论自洽。
2. **canonical 序列化**：小端定长整数、变长序列 u64 长度前缀、字段顺序即 encode/decode
   顺序、尾部多余字节报错——与 ADR-NEW-19 方向一致，但正式存档格式仍属债务 #11
   （decision-log），随 ADR-NEW-19 编码器定案。
3. **预算滚动状态**（budget_rolling）按 EC-03 现文入档；生命周期待决（债务 #20，
   ADR-NEW-14 修订定案保留或删除）。
4. **在途任务口径**：请求携带 request_id + base_network_version，"保存并恢复"已实装并通过
   强等价（ADR-NEW-16 定案 / ADR-NEW-20 单 tick 原子搜索口径下成立）。
5. 迁移策略仅验证"未知版本可诊断拒绝"；跨版本迁移本身待实际 Schema v2 需求出现后设计。

## 复现

```
cd spikes/save-restore-spike
cargo test        # 9 passed
```

测试内含黄金行为断言：7 tick 后 tick==7、三高水位随分配增长、读档后 stateful 流续列与直接运行一致。
