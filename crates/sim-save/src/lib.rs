//! CitySim-rs sim-save（Stage 1 T7）：存档/读档实装（EC-03 存档最小集 + 强读档等价）。
//!
//! 迁移自 spikes/save-restore-spike（Stage 0 Step 5 验证原型），升级点：
//! - **摘要**：FNV-1a 占位 → BLAKE3-256（sim-digest，ADR-NEW-19 已裁决采纳；
//!   DIGEST_VERSION 入摘要信封）；
//! - **编码**：手写小端布局 → sim-codec CanonicalWriter（canonical 规则 +
//!   domain separation：module="sim-save"、kind="Save"，ADR-NEW-19 顶层 Schema 独立）；
//! - **错误**：String → sim-error（SAV-0001 存档损坏 / SAV-0002 Schema 不兼容，
//!   error-code-spec.md）；
//! - **预算让步入档（G3/债务 #23）**：budget_yield_count、stale_recompute_count
//!   进入 EC-03 预算与降级类并参与摘要。
//!
//! 文件布局：canonical 信封（CSIMCOD1|sim-save|Save）+ 字段流 + 32 字节 BLAKE3 摘要
//! （摘要=BLAKE3(codec 字节流 | DIGEST_VERSION)，尾字节之外一概不收）。
//! 语义不变量：存档点仅 tick 边界；在途任务"保存并恢复"（ADR-NEW-16/20）；
//! 高水位拒绝回退；强读档等价（EC-11 R2 强形式）。
#![forbid(unsafe_code)]

use sim_codec::CanonicalWriter;
use sim_error::{codes, ErrorClass, ErrorCode, SimError, SimErrorContext};
use sim_rng::Stream64;

pub const SCHEMA_VERSION: u16 = 1;
pub const DETERMINISM_PROTOCOL_VERSION: u16 = 1;
pub use sim_rng::RNG_PROTOCOL_VERSION;
pub use sim_digest::DIGEST_VERSION;
pub use sim_digest::Digest256;

const ENVELOPE_MODULE: &str = "sim-save";
const ENVELOPE_KIND: &str = "Save";
const SAVE_BODY_KIND: &str = "SaveBody";

// ---------------- 基础结构（EC-03 各类字段载体） ----------------

/// 身份类：AgentId / 寻路请求 ID / 路网事务 ID 三类分配器高水位（ADR-NEW-20）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allocators {
    pub agent_high_water: u64,
    pub request_high_water: u64,
    pub tx_high_water: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent {
    pub id: u64,
    pub pos_mm: i64,
    pub target_mm: i64,
    pub path_budget: u64,
}

/// 在途寻路请求（在途任务类：请求 ID、AgentId、基准路网版本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathRequest {
    pub request_id: u64,
    pub agent_id: u64,
    pub base_network_version: u64,
    pub dest_mm: i64,
}

/// 未发布路网事务（简化为单边代价变更；完整版本见 spikes/network-tx-spike）。
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

// ---------------- 模拟状态（权威，七类最小集） ----------------

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
    // 在途任务（保存并恢复）
    pub request_queue: Vec<PathRequest>,
    pub pending_txs: Vec<PendingTx>,
    // 预算与降级（含 G3 让步状态，参与摘要）
    pub degrade_level: u8,
    pub budget_rolling: u64,      // 生命周期待决（债务 #20 / ADR-NEW-14 修订）
    pub budget_yield_count: u64,  // 预算让步计数（ADR-NEW-20/债务 #23）
    pub stale_recompute_count: u64, // 过期重算计数（与让步计数分离，双计数器）
    // 模拟主体
    pub agents: Vec<Agent>,
    // 非权威运行相位（不入档）：true 表示 tick 执行中，禁止存档
    #[doc(hidden)]
    pub in_tick: bool,
}

fn ctx() -> SimErrorContext {
    SimErrorContext {
        tick: 0,
        entity_id: 0,
        system: ENVELOPE_MODULE,
    }
}

fn save_err(code: ErrorCode, message: &'static str) -> SimError {
    SimError::new(ErrorClass::Fatal, code, message, ctx())
}

impl SimState {
    pub fn new(world_id: u64, global_seed: u64) -> Self {
        SimState {
            schema_version: SCHEMA_VERSION,
            world_id,
            tick: 0,
            determinism_protocol_version: DETERMINISM_PROTOCOL_VERSION,
            alloc: Allocators::default(),
            global_seed,
            rng_protocol_version: RNG_PROTOCOL_VERSION,
            streams: vec![Stream64::restore((
                global_seed ^ 0xA5A5_A5A5_A5A5_A5A5,
                0,
            ))],
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
            budget_yield_count: 0,
            stale_recompute_count: 0,
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

    /// 一个确定性 tick：发布 1 笔事务、处理 1 个请求、每 Agent 一次随机游走。
    /// 全部按稳定顺序（队列 FIFO、agents 按 id 升序）；无时钟、无浮点、无无序迭代。
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

        // 2) 处理至多 1 个在途寻路请求（FIFO 稳定序）。
        //    降级期（degrade_level>0）奇数 tick 让步——确定性规则示例，
        //    计入 budget_yield_count（真实预算机制随 2.7/ADR-NEW-20 落地）。
        if !(self.degrade_level > 0 && self.tick % 2 == 1) {
            if let Some(req) = self.request_queue.first().cloned() {
                self.request_queue.remove(0);
                // 过期结果计数（双计数器分离：stale ≠ yield）
                if req.base_network_version != self.network_version {
                    self.stale_recompute_count += 1;
                }
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
        } else {
            self.budget_yield_count += 1;
        }

        // 3) 每 Agent：一次有状态流随机游走（按 id 升序）
        let n_streams = self.streams.len();
        for i in 0..self.agents.len() {
            let draw = self.streams[i % n_streams].next_u64();
            let delta = (draw % 2001) as i64 - 1000;
            let a = &mut self.agents[i];
            a.pos_mm = a.pos_mm.wrapping_add(delta);
            // 累计工作量预算（确定性单位：每 Agent 1 单位/tick）
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

    // ---------------- canonical 编码（sim-codec） ----------------

    /// 状态正文的 canonical 字节流（无摘要尾）。字段顺序即契约。
    pub fn encode_body(&self) -> Vec<u8> {
        let mut w = CanonicalWriter::new(ENVELOPE_MODULE, ENVELOPE_KIND);
        w.u16(self.schema_version);
        w.u16(self.determinism_protocol_version);
        w.u16(self.rng_protocol_version);
        w.u64(self.world_id);
        w.u64(self.tick);
        w.u64(self.alloc.agent_high_water);
        w.u64(self.alloc.request_high_water);
        w.u64(self.alloc.tx_high_water);
        w.u64(self.global_seed);
        w.seq_len(self.streams.len());
        for s in &self.streams {
            let (seed, counter) = s.save();
            w.u64(seed);
            w.u64(counter);
        }
        w.u64(self.network_version);
        w.seq_len(self.edges.len());
        for e in &self.edges {
            w.u64(e.id);
            w.u64(e.a);
            w.u64(e.b);
            w.i64(e.cost);
        }
        w.seq_len(self.request_queue.len());
        for r in &self.request_queue {
            w.u64(r.request_id);
            w.u64(r.agent_id);
            w.u64(r.base_network_version);
            w.i64(r.dest_mm);
        }
        w.seq_len(self.pending_txs.len());
        for t in &self.pending_txs {
            w.u64(t.tx_id);
            w.u64(t.edge);
            w.i64(t.new_cost);
        }
        w.seq_len(self.agents.len());
        for a in &self.agents {
            w.u64(a.id);
            w.i64(a.pos_mm);
            w.i64(a.target_mm);
            w.u64(a.path_budget);
        }
        w.u8(self.degrade_level);
        w.u64(self.budget_rolling);
        w.u64(self.budget_yield_count);
        w.u64(self.stale_recompute_count);
        w.into_bytes()
    }

    /// 存档（仅 tick 边界；EC-03 条文）。返回 canonical 文件字节 + 存档点摘要。
    pub fn save(&self) -> Result<(Vec<u8>, sim_digest::Digest256), SimError> {
        if self.in_tick {
            return Err(save_err(codes::SAVE_CORRUPT, "save point must be at a tick boundary"));
        }
        let body = self.encode_body();
        let d = sim_digest::digest(ENVELOPE_MODULE, SAVE_BODY_KIND, &body);
        let mut file = body;
        file.extend_from_slice(&d.0);
        Ok((file, d))
    }

    /// 读档 + 一致性检查（信封/Schema/摘要/尾字节全部校验，失败可诊断）。
    pub fn load(raw: &[u8]) -> Result<SimState, SimError> {
        if raw.len() < 32 {
            return Err(save_err(codes::SAVE_CORRUPT, "truncated save: shorter than digest"));
        }
        let (body, tail) = raw.split_at(raw.len() - 32);
        let d = sim_digest::digest(ENVELOPE_MODULE, SAVE_BODY_KIND, body);
        if d.0 != tail {
            return Err(save_err(
                codes::SAVE_CORRUPT,
                "save body digest mismatch (corrupt or tampered)",
            ));
        }
        SimState::decode_body(body)
    }

    fn decode_body(body: &[u8]) -> Result<SimState, SimError> {
        let mut r = Reader::new(body)?;
        // 信封=域分隔头（CSIMCOD1|sim-save|Save），即"是否 CitySim 存档"的判据
        let ver = r.str()?;
        if ver != sim_codec::CODEC_VERSION {
            return Err(save_err(codes::SAVE_CORRUPT, "not a CitySim save (bad codec envelope)"));
        }
        let module = r.str()?;
        let kind = r.str()?;
        if module != ENVELOPE_MODULE || kind != ENVELOPE_KIND {
            return Err(save_err(codes::SAVE_CORRUPT, "not a CitySim save (bad codec envelope)"));
        }
        let schema_version = r.u16()?;
        if schema_version != SCHEMA_VERSION {
            // 迁移失败必须可诊断（EC-03 设计要求）
            return Err(save_err(
                codes::SAVE_SCHEMA_UNSUPPORTED,
                "unsupported schema version: migration required",
            ));
        }
        let mut s = SimState {
            schema_version,
            determinism_protocol_version: r.u16()?,
            rng_protocol_version: r.u16()?,
            world_id: r.u64()?,
            tick: r.u64()?,
            alloc: Allocators {
                agent_high_water: r.u64()?,
                request_high_water: r.u64()?,
                tx_high_water: r.u64()?,
            },
            global_seed: r.u64()?,
            streams: Vec::new(),
            network_version: 0,
            edges: Vec::new(),
            request_queue: Vec::new(),
            pending_txs: Vec::new(),
            degrade_level: 0,
            budget_rolling: 0,
            budget_yield_count: 0,
            stale_recompute_count: 0,
            agents: Vec::new(),
            in_tick: false,
        };
        let n = r.seq_len()?;
        for _ in 0..n {
            let seed = r.u64()?;
            let counter = r.u64()?;
            s.streams.push(Stream64::restore((seed, counter)));
        }
        s.network_version = r.u64()?;
        let n = r.seq_len()?;
        for _ in 0..n {
            s.edges.push(Edge {
                id: r.u64()?,
                a: r.u64()?,
                b: r.u64()?,
                cost: r.i64()?,
            });
        }
        let n = r.seq_len()?;
        for _ in 0..n {
            s.request_queue.push(PathRequest {
                request_id: r.u64()?,
                agent_id: r.u64()?,
                base_network_version: r.u64()?,
                dest_mm: r.i64()?,
            });
        }
        let n = r.seq_len()?;
        for _ in 0..n {
            s.pending_txs.push(PendingTx {
                tx_id: r.u64()?,
                edge: r.u64()?,
                new_cost: r.i64()?,
            });
        }
        let n = r.seq_len()?;
        for _ in 0..n {
            s.agents.push(Agent {
                id: r.u64()?,
                pos_mm: r.i64()?,
                target_mm: r.i64()?,
                path_budget: r.u64()?,
            });
        }
        s.degrade_level = r.u8()?;
        s.budget_rolling = r.u64()?;
        s.budget_yield_count = r.u64()?;
        s.stale_recompute_count = r.u64()?;
        // 不兼容/多余数据不能静默丢弃
        if r.p != body.len() {
            return Err(save_err(
                codes::SAVE_CORRUPT,
                "trailing bytes after save body: refuse to ignore",
            ));
        }
        Ok(s)
    }
}

/// 状态摘要（GlobalDigest 单点口径：正文 canonical 字节 + 摘要信封）。
pub fn state_digest(s: &SimState) -> sim_digest::Digest256 {
    sim_digest::digest(ENVELOPE_MODULE, SAVE_BODY_KIND, &s.encode_body())
}

// ---------------- canonical 读取器（信封字段序镜像 sim-codec） ----------------

struct Reader<'a> {
    raw: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn new(raw: &'a [u8]) -> Result<Self, SimError> {
        Ok(Reader { raw, p: 0 })
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], SimError> {
        if self.p + n > self.raw.len() {
            return Err(save_err(codes::SAVE_CORRUPT, "truncated save body"));
        }
        let v = &self.raw[self.p..self.p + n];
        self.p += n;
        Ok(v)
    }

    fn u8(&mut self) -> Result<u8, SimError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, SimError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, SimError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64, SimError> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn seq_len(&mut self) -> Result<usize, SimError> {
        Ok(self.u64()? as usize)
    }

    fn str(&mut self) -> Result<String, SimError> {
        let n = self.u64()? as usize;
        let b = self.take(n)?;
        String::from_utf8(b.to_vec())
            .map_err(|_| save_err(codes::SAVE_CORRUPT, "non-utf8 length-prefixed string"))
    }
}

// ---------------- 测试（迁移自 Spike，摘要口径升级为 BLAKE3） ----------------

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
        assert_eq!(r.schema_version, SCHEMA_VERSION); // 元数据
        assert_eq!(r.tick, 7);
        assert_eq!(r.alloc, s.alloc); // 身份（三类高水位）
        assert_eq!(r.global_seed, 0x1234_5678_9ABC_DEF0); // 随机数
        assert!(!r.streams.is_empty());
        assert_eq!(r.network_version, s.network_version); // 路网
        assert_eq!(r.request_queue, s.request_queue); // 在途任务（保存并恢复）
        assert_eq!(r.pending_txs, s.pending_txs);
        assert_eq!(r.degrade_level, s.degrade_level); // 预算与降级（含 G3 让步计数）
        assert_eq!(r.budget_yield_count, s.budget_yield_count);
        assert_eq!(r.stale_recompute_count, s.stale_recompute_count);
        assert_eq!(state_digest(&r), digest); // 校验
    }

    /// 存档点只能位于 tick 边界。
    #[test]
    fn save_rejects_non_tick_boundary() {
        let s = scenario(1);
        let mut mid = s.clone();
        mid.in_tick = true;
        assert!(mid.save().is_err());
    }

    /// 强读档等价（EC-11 R2 强形式）：N=40 存档读档跑至 M=100 == 直接跑至 100。
    #[test]
    fn strong_readback_equivalence() {
        let (n, m) = (40u64, 100u64);
        let mut a = scenario(0xDEAD_BEEF_CAFE_0001);
        let mut b = a.clone();
        for _ in 0..n {
            a.run_tick();
            b.run_tick();
        }
        assert!(
            !b.request_queue.is_empty() || !b.pending_txs.is_empty(),
            "save point must have in-flight work"
        );
        let (bytes, _) = b.save().unwrap();
        let mut b2 = SimState::load(&bytes).unwrap();
        for _ in n..m {
            a.run_tick();
            b2.run_tick();
        }
        assert_eq!(state_digest(&a), state_digest(&b2), "strong readback equivalence violated");
        assert_eq!(a, b2);
    }

    /// 让步路径强读档专项（Stage 2B 硬门禁，第二轮评审条件⑥）：
    /// 存档点处于让步态（budget_yield_count > 0）时，强读档等价仍成立且让步计数延续。
    #[test]
    fn yield_state_strong_readback() {
        let mut a = scenario(0x1EE7_5EED_0000_0007);
        a.budget_rolling = 10_001; // 触发降级 → 降级期奇数 tick 让步
        let mut n = 0u64;
        while a.budget_yield_count == 0 && n < 10 {
            a.run_tick();
            n += 1;
        }
        assert!(a.budget_yield_count > 0, "must reach yield state before save");
        let saved_yield = a.budget_yield_count;
        let b = a.clone();
        let (bytes, _) = b.save().unwrap();
        let r = SimState::load(&bytes).unwrap();
        assert_eq!(r.budget_yield_count, saved_yield, "让步计数必须入档（G3）");
        let mut b2 = r;
        for _ in 0..60 {
            a.run_tick();
            b2.run_tick();
        }
        assert_eq!(state_digest(&a), state_digest(&b2), "让步态强读档等价被破坏");
        assert!(a.budget_yield_count >= saved_yield, "让步计数延续");
    }

    /// 弱形式（政策若改"丢弃并重新入队"则以此为准）：两次独立存档-读档运行一致。
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

    /// 有状态随机流回读后续列不断裂（16 字节/流入档）。
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
            (0..3).map(|_| t.streams[0].next_u64()).collect()
        };
        let next_restored: Vec<u64> = (0..3).map(|_| r.streams[0].next_u64()).collect();
        assert_eq!(next_direct, next_restored, "stream continuation must match after load");
    }

    /// 篡改存档任一字节 → 一致性检查拒绝（BLAKE3 摘要比对）。
    #[test]
    fn tampered_save_is_rejected() {
        let mut s = scenario(5);
        s.run_tick();
        let (mut bytes, _) = s.save().unwrap();
        let idx = bytes.len() / 2;
        bytes[idx] ^= 0xFF;
        assert!(SimState::load(&bytes).is_err(), "tampered save must fail consistency check");
    }

    /// 未知 Schema 版本必须可诊断地失败（SAV-0002）。
    #[test]
    fn unknown_schema_is_diagnosable() {
        let s = scenario(5);
        let (mut bytes, _) = s.save().unwrap();
        // schema_version 是信封三串后的第一个 u16（低字节在前）
        // 信封 = 3×(u64 长度前缀 8B + 串体) = (8+8) + (8+8) + (8+4) = 44 字节
        let off = 8 + 8 + 8 + 8 + 8 + 4;
        bytes[off] = 99;
        // 摘要随之失效——先修摘要以单独验证 schema 检查
        let body = &bytes[..bytes.len() - 32];
        let d = sim_digest::digest(ENVELOPE_MODULE, SAVE_BODY_KIND, body);
        let n = bytes.len();
        bytes[n - 32..].copy_from_slice(&d.0);
        let err = SimState::load(&bytes).unwrap_err();
        assert_eq!(err.code, codes::SAVE_SCHEMA_UNSUPPORTED, "must name the schema problem");
    }

    /// 尾部多余字节拒绝。
    #[test]
    fn trailing_bytes_rejected() {
        let s = scenario(5);
        let (mut bytes, _) = s.save().unwrap();
        bytes.push(0);
        assert!(SimState::load(&bytes).is_err());
    }

    /// G3：让步/过期计数为独立字段（双计数器分离，债务 #23）。
    #[test]
    fn yield_and_stale_counters_are_separate() {
        let mut s = scenario(3);
        for _ in 0..12 {
            s.run_tick();
        }
        // 两计数各自有界、字段独立入档
        let (bytes, _) = s.save().unwrap();
        let r = SimState::load(&bytes).unwrap();
        assert_eq!(r.budget_yield_count, s.budget_yield_count);
        assert_eq!(r.stale_recompute_count, s.stale_recompute_count);
    }
}
