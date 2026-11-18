//! save-restore-spike — Stage 0 Step 5：EC-03 存档最小集 + 强读档等价验证原型。
//!
//! 目标（主文档 3.3 EC-03）：
//! - 最小兼容核心字段候选集的完备性（7 类：元数据/身份/随机数/路网/在途任务/预算与降级/校验）；
//! - 存档点仅 tick 边界；在途任务"保存并恢复"（ADR-NEW-16 定案）；
//! - 读档后一致性检查 + 摘要比对；强读档等价（第 N tick 存档读档跑至 M == 直接跑至 M）。
//!
//! ⚠ 摘要占位说明：本原型用 FNV-1a 64 位对 canonical 字节求"状态摘要"作为
//! GlobalDigest 的**占位实现**（ADR-NEW-19 尚为 Proposed，BLAKE3-256 未裁决）。
//! 等价测试内部自洽：两侧用同一占位算法。正式 GlobalDigest 落定后替换该函数即可。
//!
//! canonical 序列化口径（与 ADR-NEW-19 方向一致）：小端定长整数、变长序列 u64 长度前缀、
//! 字段顺序即本文件 `encode`/`decode` 顺序；尾部多余字节报错（不静默丢弃）。

#![forbid(unsafe_code)]

pub const MAGIC: [u8; 4] = *b"CSAV";
pub const SCHEMA_VERSION: u16 = 1;
pub const DETERMINISM_PROTOCOL_VERSION: u16 = 1;
pub const RNG_PROTOCOL_VERSION: u16 = 1;

// ---------------- 基础结构 ----------------

/// AgentId 分配器 + 请求/事务 ID 分配器高水位（EC-03 身份类、ADR-NEW-20）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allocators {
    pub agent_high_water: u64,
    pub request_high_water: u64,
    pub tx_high_water: u64,
}

/// 有状态随机流（ADR-NEW-22 Stream64 口径：seed + counter，16 字节存档）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stream64 {
    pub seed: u64,
    pub counter: u64,
}

impl Stream64 {
    // 名称即"流的下一值"，非迭代器协议；实现 Iterator 反而误导（存档语义见 save/restore）。
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.seed.wrapping_add(self.counter);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent {
    pub id: u64,
    pub pos_mm: i64,
    pub target_mm: i64,
    pub path_budget: u64,
}

/// 在途寻路请求（EC-03 在途任务类：请求 ID、AgentId、基准路网版本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathRequest {
    pub request_id: u64,
    pub agent_id: u64,
    pub base_network_version: u64,
    pub dest_mm: i64,
}

/// 未发布路网事务（简化为单边代价变更；Spike C 原型有完整版本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingTx {
    pub tx_id: u64,
    pub edge: u64,
    pub new_cost: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub id: u64,
    pub a: u64,
    pub b: u64,
    pub cost: i64,
}

// ---------------- 模拟状态（权威） ----------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimState {
    // 元数据
    pub schema_version: u16,
    pub world_id: u64,
    pub tick: u64,
    pub determinism_protocol_version: u16,
    // 身份
    pub alloc: Allocators,
    // 随机数
    pub global_seed: u64,
    pub rng_protocol_version: u16,
    pub streams: Vec<Stream64>, // 有状态流必须存档（EC-02/ADR-NEW-22）
    // 路网
    pub network_version: u64,
    pub edges: Vec<Edge>,
    // 在途任务
    pub request_queue: Vec<PathRequest>,
    pub pending_txs: Vec<PendingTx>,
    // 预算与降级
    pub degrade_level: u8,
    pub budget_rolling: u64, // 生命周期待决（债务 #20 / ADR-NEW-14 修订）
    // 模拟主体
    pub agents: Vec<Agent>,
    // 非权威运行相位（不入档）：true 表示 tick 执行中，禁止存档
    #[doc(hidden)]
    pub in_tick: bool,
}

impl SimState {
    pub fn new(world_id: u64, global_seed: u64) -> Self {
        SimState {
            schema_version: SCHEMA_VERSION,
            world_id,
            tick: 0,
            determinism_protocol_version: DETERMINISM_PROTOCOL_VERSION,
            alloc: Allocators { agent_high_water: 0, request_high_water: 0, tx_high_water: 0 },
            global_seed,
            rng_protocol_version: RNG_PROTOCOL_VERSION,
            streams: vec![Stream64 { seed: global_seed ^ 0xA5A5_A5A5_A5A5_A5A5, counter: 0 }],
            network_version: 1,
            edges: vec![
                Edge { id: 0, a: 0, b: 1, cost: 100 },
                Edge { id: 1, a: 1, b: 2, cost: 100 },
                Edge { id: 2, a: 2, b: 3, cost: 100 },
            ],
            request_queue: Vec::new(),
            pending_txs: Vec::new(),
            degrade_level: 0,
            budget_rolling: 0,
            agents: Vec::new(),
            in_tick: false,
        }
    }

    pub fn spawn_agent(&mut self, pos_mm: i64, target_mm: i64) -> u64 {
        self.alloc.agent_high_water += 1;
        let id = self.alloc.agent_high_water;
        self.agents.push(Agent { id, pos_mm, target_mm, path_budget: 0 });
        id
    }

    pub fn enqueue_request(&mut self, agent_id: u64, dest_mm: i64) -> u64 {
        self.alloc.request_high_water += 1;
        let id = self.alloc.request_high_water;
        self.request_queue.push(PathRequest {
            request_id: id,
            agent_id,
            base_network_version: self.network_version,
            dest_mm,
        });
        id
    }

    pub fn enqueue_tx(&mut self, edge: u64, new_cost: i64) -> u64 {
        self.alloc.tx_high_water += 1;
        let id = self.alloc.tx_high_water;
        self.pending_txs.push(PendingTx { tx_id: id, edge, new_cost });
        id
    }

    /// 一个确定性 tick：发布 1 笔事务、处理 2 个请求、每 Agent 一次随机游走。
    /// 全部按稳定顺序（队列 FIFO、agents 按 id 升序）。
    pub fn run_tick(&mut self) {
        assert!(!self.in_tick, "nested tick");
        self.in_tick = true;

        // 1) tick 边界发布一笔未发布路网事务（原子：整笔生效）
        if let Some(tx) = self.pending_txs.first().cloned() {
            for e in self.edges.iter_mut() {
                if e.id == tx.edge {
                    e.cost = tx.new_cost;
                }
            }
            self.network_version += 1;
            self.pending_txs.remove(0);
        }

        // 2) 处理至多 1 个在途寻路请求（FIFO 稳定序；结果落 path_budget）
        for _ in 0..1 {
            if self.request_queue.is_empty() {
                break;
            }
            let req = self.request_queue.remove(0);
            // 确定性代价计算：边代价按 id 升序求和 + 目标
            let mut total: i64 = 0;
            let mut es: Vec<Edge> = self.edges.clone();
            es.sort_by_key(|e| e.id);
            for e in &es {
                total = total.wrapping_add(e.cost.wrapping_add(req.dest_mm));
            }
            if let Some(a) = self.agents.iter_mut().find(|a| a.id == req.agent_id) {
                a.path_budget = total as u64 ^ req.request_id ^ req.base_network_version;
            }
        }

        // 3) 每 Agent：一次有状态流随机游走（按 id 升序）
        let n_streams = self.streams.len();
        for i in 0..self.agents.len() {
            let draw = self.streams[i % n_streams].next();
            let delta = (draw % 2001) as i64 - 1000;
            let a = &mut self.agents[i];
            a.pos_mm = a.pos_mm.wrapping_add(delta);
            // 累计工作量预算（确定性单位：每 Agent 1 单位/ tick）
            self.budget_rolling = self.budget_rolling.wrapping_add(1);
        }

        // 4) 确定性降级信号（只依赖工作量单位，见 2.7/EC-04/ADR-NEW-14）
        self.degrade_level = if self.budget_rolling > 10_000 { 1 } else { 0 };

        // 5) 持续产生在途工作（确定性节奏 2 请求/tick + 1 事务/2 tick），
        //    保证任意存档点都存在"保存并恢复"要携带的在途任务。
        let hw = self.alloc.agent_high_water.max(1);
        for k in 0..2u64 {
            let aid = 1 + ((self.tick + k) % hw);
            self.enqueue_request(aid, ((self.tick + k) as i64).wrapping_mul(7));
        }
        if self.tick.is_multiple_of(2) {
            self.enqueue_tx(self.tick % 3, (self.tick as i64).wrapping_mul(3));
        }

        self.tick += 1;
        self.in_tick = false;
    }

    // ---------------- canonical 序列化 ----------------

    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(1024);
        b.extend_from_slice(&MAGIC);
        b.extend_from_slice(&self.schema_version.to_le_bytes());
        b.extend_from_slice(&self.determinism_protocol_version.to_le_bytes());
        b.extend_from_slice(&self.rng_protocol_version.to_le_bytes());
        b.extend_from_slice(&self.world_id.to_le_bytes());
        b.extend_from_slice(&self.tick.to_le_bytes());
        b.extend_from_slice(&self.alloc.agent_high_water.to_le_bytes());
        b.extend_from_slice(&self.alloc.request_high_water.to_le_bytes());
        b.extend_from_slice(&self.alloc.tx_high_water.to_le_bytes());
        b.extend_from_slice(&self.global_seed.to_le_bytes());
        b.extend_from_slice(&(self.streams.len() as u64).to_le_bytes());
        for s in &self.streams {
            b.extend_from_slice(&s.seed.to_le_bytes());
            b.extend_from_slice(&s.counter.to_le_bytes());
        }
        b.extend_from_slice(&self.network_version.to_le_bytes());
        b.extend_from_slice(&(self.edges.len() as u64).to_le_bytes());
        for e in &self.edges {
            b.extend_from_slice(&e.id.to_le_bytes());
            b.extend_from_slice(&e.a.to_le_bytes());
            b.extend_from_slice(&e.b.to_le_bytes());
            b.extend_from_slice(&e.cost.to_le_bytes());
        }
        b.extend_from_slice(&(self.request_queue.len() as u64).to_le_bytes());
        for r in &self.request_queue {
            b.extend_from_slice(&r.request_id.to_le_bytes());
            b.extend_from_slice(&r.agent_id.to_le_bytes());
            b.extend_from_slice(&r.base_network_version.to_le_bytes());
            b.extend_from_slice(&r.dest_mm.to_le_bytes());
        }
        b.extend_from_slice(&(self.pending_txs.len() as u64).to_le_bytes());
        for t in &self.pending_txs {
            b.extend_from_slice(&t.tx_id.to_le_bytes());
            b.extend_from_slice(&t.edge.to_le_bytes());
            b.extend_from_slice(&t.new_cost.to_le_bytes());
        }
        b.extend_from_slice(&(self.agents.len() as u64).to_le_bytes());
        for a in &self.agents {
            b.extend_from_slice(&a.id.to_le_bytes());
            b.extend_from_slice(&a.pos_mm.to_le_bytes());
            b.extend_from_slice(&a.target_mm.to_le_bytes());
            b.extend_from_slice(&a.path_budget.to_le_bytes());
        }
        b.extend_from_slice(&self.degrade_level.to_le_bytes());
        b.extend_from_slice(&self.budget_rolling.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes()); // 校验类：摘要槽（占位 0，save() 时回填）
        b
    }

    pub fn decode(raw: &[u8]) -> Result<SimState, String> {
        let mut p = 0usize;
        let take = |n: usize, p: &mut usize| -> Result<Vec<u8>, String> {
            if *p + n > raw.len() {
                return Err(format!("truncated: need {} bytes at {}", n, p));
            }
            let v = raw[*p..*p + n].to_vec();
            *p += n;
            Ok(v)
        };
        let u16le = |v: Vec<u8>| u16::from_le_bytes([v[0], v[1]]);
        let u64le = |v: Vec<u8>| u64::from_le_bytes(v[..8].try_into().unwrap());
        let i64le = |v: Vec<u8>| i64::from_le_bytes(v[..8].try_into().unwrap());

        let magic = take(4, &mut p)?;
        if magic != MAGIC {
            return Err("bad magic: not a CitySim save".into());
        }
        let schema = u16le(take(2, &mut p)?);
        if schema != SCHEMA_VERSION {
            // 迁移失败必须可诊断（EC-03 设计要求）
            return Err(format!(
                "unsupported schema version {} (this build reads {}): migration required",
                schema, SCHEMA_VERSION
            ));
        }
        let mut s = SimState {
            schema_version: schema,
            determinism_protocol_version: u16le(take(2, &mut p)?),
            rng_protocol_version: u16le(take(2, &mut p)?),
            world_id: u64le(take(8, &mut p)?),
            tick: u64le(take(8, &mut p)?),
            alloc: Allocators {
                agent_high_water: u64le(take(8, &mut p)?),
                request_high_water: u64le(take(8, &mut p)?),
                tx_high_water: u64le(take(8, &mut p)?),
            },
            global_seed: u64le(take(8, &mut p)?),
            streams: Vec::new(),
            network_version: 0, // 占位：streams 之后读取（与 encode 字段序一致）
            edges: Vec::new(),
            request_queue: Vec::new(),
            pending_txs: Vec::new(),
            degrade_level: 0,
            budget_rolling: 0,
            agents: Vec::new(),
            in_tick: false,
        };
        let n = u64le(take(8, &mut p)?) as usize;
        for _ in 0..n {
            s.streams.push(Stream64 { seed: u64le(take(8, &mut p)?), counter: u64le(take(8, &mut p)?) });
        }
        s.network_version = u64le(take(8, &mut p)?);
        let n = u64le(take(8, &mut p)?) as usize;
        for _ in 0..n {
            s.edges.push(Edge {
                id: u64le(take(8, &mut p)?),
                a: u64le(take(8, &mut p)?),
                b: u64le(take(8, &mut p)?),
                cost: i64le(take(8, &mut p)?),
            });
        }
        let n = u64le(take(8, &mut p)?) as usize;
        for _ in 0..n {
            s.request_queue.push(PathRequest {
                request_id: u64le(take(8, &mut p)?),
                agent_id: u64le(take(8, &mut p)?),
                base_network_version: u64le(take(8, &mut p)?),
                dest_mm: i64le(take(8, &mut p)?),
            });
        }
        let n = u64le(take(8, &mut p)?) as usize;
        for _ in 0..n {
            s.pending_txs.push(PendingTx {
                tx_id: u64le(take(8, &mut p)?),
                edge: u64le(take(8, &mut p)?),
                new_cost: i64le(take(8, &mut p)?),
            });
        }
        let n = u64le(take(8, &mut p)?) as usize;
        for _ in 0..n {
            s.agents.push(Agent {
                id: u64le(take(8, &mut p)?),
                pos_mm: i64le(take(8, &mut p)?),
                target_mm: i64le(take(8, &mut p)?),
                path_budget: u64le(take(8, &mut p)?),
            });
        }
        s.degrade_level = take(1, &mut p)?[0];
        s.budget_rolling = u64le(take(8, &mut p)?);
        let digest = u64le(take(8, &mut p)?);

        // 不兼容/多余数据不能静默丢弃
        if p != raw.len() {
            return Err(format!("trailing {} bytes after save body: refuse to ignore", raw.len() - p));
        }
        // 读档一致性检查：重算摘要并比对（EC-03 条文）
        let re = fnv1a64(&raw[..raw.len() - 8]);
        if re != digest {
            return Err(format!("digest mismatch after load: stored {:#018x}, recomputed {:#018x}", digest, re));
        }
        Ok(s)
    }

    /// 存档（仅 tick 边界；EC-03 条文）。返回 canonical 字节 + 存档点摘要。
    pub fn save(&self) -> Result<(Vec<u8>, u64), String> {
        if self.in_tick {
            return Err("save point must be at a tick boundary".into());
        }
        let mut bytes = self.encode();
        let d = fnv1a64(&bytes[..bytes.len() - 8]); // 正文（摘要槽 0 占位已被排除）
        let tail = bytes.len() - 8;
        bytes[tail..].copy_from_slice(&d.to_le_bytes()); // 回填摘要
        Ok((bytes, d))
    }

    /// 读档 + 一致性检查（decode 内含摘要比对）。
    pub fn load(raw: &[u8]) -> Result<SimState, String> {
        SimState::decode(raw)
    }
}

/// GlobalDigest 占位：FNV-1a 64 over canonical 字节（不含摘要尾字段）。
/// ⚠ ADR-NEW-19 裁决后替换为 BLAKE3-256 + digest_version；本函数是唯一替换点。
pub fn state_digest(s: &SimState) -> u64 {
    // encode() 尾部摘要槽恒为 0 占位，故对正文（去掉末 8 字节）求 FNV 即状态摘要。
    let full = s.encode();
    fnv1a64(&full[..full.len() - 8])
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

// ---------------- 测试 ----------------

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario(seed: u64) -> SimState {
        let mut s = SimState::new(7, seed);
        for i in 0..5 {
            s.spawn_agent(i * 1000, i * 2000 + 500);
        }
        for a in 1..=5u64 {
            s.enqueue_request(a, a as i64 * 333);
        }
        s.enqueue_tx(1, 250);
        s.enqueue_tx(2, 500);
        s
    }

    /// EC-03 七类字段全部入档且可完整回读（逐类覆盖）。
    #[test]
    fn save_set_covers_ec03_categories() {
        let mut s = scenario(0x1234_5678_9ABC_DEF0);
        for _ in 0..7 {
            s.run_tick();
        }
        let (bytes, digest) = s.save().unwrap();
        let r = SimState::load(&bytes).unwrap();
        assert_eq!(r, s, "roundtrip must preserve every field");
        // 逐类抽查
        assert_eq!(r.schema_version, SCHEMA_VERSION); // 元数据
        assert_eq!(r.tick, 7);
        assert_eq!(r.alloc.agent_high_water, s.alloc.agent_high_water); // 身份（三类高水位）
        assert_eq!(r.alloc.request_high_water, s.alloc.request_high_water);
        assert_eq!(r.alloc.tx_high_water, s.alloc.tx_high_water);
        assert_eq!(r.global_seed, 0x1234_5678_9ABC_DEF0); // 随机数
        assert!(!r.streams.is_empty());
        assert_eq!(r.network_version, s.network_version); // 路网
        assert_eq!(r.request_queue, s.request_queue); // 在途任务（保存并恢复）
        assert_eq!(r.pending_txs, s.pending_txs);
        assert_eq!(r.degrade_level, s.degrade_level); // 预算与降级
        assert_eq!(r.budget_rolling, s.budget_rolling);
        assert_eq!(state_digest(&r), digest); // 校验
    }

    /// 存档点只能位于 tick 边界。
    #[test]
    fn save_rejects_non_tick_boundary() {
        let s = scenario(1);
        let mut mid = s.clone();
        mid.in_tick = true; // 模拟 tick 执行中
        assert!(mid.save().is_err());
    }

    /// 强读档等价（EC-11 R2 强形式）：N=40 存档读档跑至 M=100 == 直接跑至 100。
    /// 保存点上保留在途请求与未发布事务（覆盖"保存并恢复"语义）。
    #[test]
    fn strong_readback_equivalence() {
        let (n, m) = (40u64, 100u64);
        let mut a = scenario(0xDEAD_BEEF_CAFE_0001);
        let mut b = a.clone();
        for _ in 0..n {
            a.run_tick();
            b.run_tick();
        }
        assert!(!b.request_queue.is_empty() || !b.pending_txs.is_empty(), "save point must have in-flight work");
        let (bytes, _) = b.save().unwrap();
        let mut b2 = SimState::load(&bytes).unwrap();
        for _ in n..m {
            a.run_tick();
            b2.run_tick();
        }
        assert_eq!(state_digest(&a), state_digest(&b2), "strong readback equivalence violated");
        assert_eq!(a, b2);
    }

    /// 弱形式（若将来改为"丢弃并重新入队"则以此为准）：两次独立存档-读档运行一致。
    #[test]
    fn weak_form_equivalence() {
        let (n, m) = (40u64, 100u64);
        let run = |save_at: u64| {
            let mut s = scenario(0xDEAD_BEEF_CAFE_0001);
            for _ in 0..save_at {
                s.run_tick();
            }
            let (bytes, _) = s.save().unwrap();
            let mut s = SimState::load(&bytes).unwrap();
            for _ in save_at..m {
                s.run_tick();
            }
            state_digest(&s)
        };
        assert_eq!(run(n), run(n + 5), "weak form: two independent save/load runs must agree");
    }

    /// 高水位语义：读档后分配的新 ID 不得复用（Agent/请求/事务三类）。
    #[test]
    fn id_high_water_prevents_reuse() {
        let mut s = scenario(99);
        s.run_tick();
        let (bytes, _) = s.save().unwrap();
        let mut r = SimState::load(&bytes).unwrap();
        let old_a = r.alloc.agent_high_water;
        let new_a = r.spawn_agent(0, 0);
        assert!(new_a > old_a, "agent id must not be reused");
        let old_q = r.alloc.request_high_water;
        let new_q = r.enqueue_request(new_a, 0);
        assert!(new_q > old_q);
        let old_t = r.alloc.tx_high_water;
        let new_t = r.enqueue_tx(0, 1);
        assert!(new_t > old_t);
    }

    /// 有状态随机流回读后续列不断裂（16 字节/流全部入档）。
    #[test]
    fn stateful_rng_roundtrip() {
        let mut s = scenario(0x0F0F_0F0F_0F0F_0F0F);
        for _ in 0..10 {
            s.run_tick();
        }
        let (bytes, _) = s.save().unwrap();
        let mut r = SimState::load(&bytes).unwrap();
        let next_direct: Vec<u64> = {
            let mut t = s.clone();
            (0..3).map(|_| t.streams[0].next()).collect()
        };
        let next_restored: Vec<u64> = (0..3).map(|_| r.streams[0].next()).collect();
        assert_eq!(next_direct, next_restored, "stream continuation must match after load");
    }

    /// 篡改存档任一字节 → 一致性检查拒绝（摘要比对）。
    #[test]
    fn tampered_save_is_rejected() {
        let mut s = scenario(5);
        s.run_tick();
        let (mut bytes, _) = s.save().unwrap();
        let idx = bytes.len() / 2;
        bytes[idx] ^= 0xFF;
        assert!(SimState::load(&bytes).is_err(), "tampered save must fail consistency check");
    }

    /// 未知 Schema 版本必须可诊断地失败（不静默丢弃、迁移失败可诊断）。
    #[test]
    fn unknown_schema_is_diagnosable() {
        let s = scenario(5);
        let (mut bytes, _) = s.save().unwrap();
        bytes[4] = 99; // schema_version 低字节
        let err = SimState::load(&bytes).unwrap_err();
        assert!(err.contains("schema"), "error must name the schema problem, got: {err}");
    }

    /// 尾部多余字节拒绝。
    #[test]
    fn trailing_bytes_rejected() {
        let s = scenario(5);
        let (mut bytes, _) = s.save().unwrap();
        bytes.push(0);
        assert!(SimState::load(&bytes).is_err());
    }
}
