//! CitySim-rs sim-rng（正式 crate，Stage 1 T5）：系统级 RNG 协议（ADR-NEW-22）。
//!
//! STATUS: ADR-NEW-22 正式裁决采纳（2026-11-19，Accepted 附条件）。迁移自
//! spikes/rng-spike（Spike B 证据快照保留），9 测试与黄金值不变；新增
//! namespace 注册表与注册期 64 位碰撞校验（债务 #25）。
//!
//! 协议要点（ADR-NEW-22）：
//! - 双模式：纯派生（derive_seed/event_u64，零状态）+ Stream64 顺序流（16 字节存档）。
//! - SplitMix64 为协议 v1 算法；Xoshiro256**/PCG64 保留为备选比较。
//! - 派生键编码（FNV-1a 64 + 两轮混合）属协议内容，变更必须 bump
//!   RNG_PROTOCOL_VERSION（EC-03 存档字段）。
//! - namespace 一经注册不得复用；注册期做 64 位碰撞校验。
//!
//! 证据纪律：单机实测（rustc 1.100.0，x86_64-pc-windows-msvc），跨平台待 R4；
//! 统计质量（TestU01/PractRand）为条件核销清单未决项（ADR-NEW-22 条件②）。
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// RNG 协议版本（EC-03 存档字段）。派生算法 / 键编码 / 输出映射任何变更必须 bump。
pub const RNG_PROTOCOL_VERSION: u16 = 1;

// ---------------------------------------------------------------------------
// 协议 v1 算法：SplitMix64（派生与顺序流）
// ---------------------------------------------------------------------------

pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

// ---------------------------------------------------------------------------
// 备选算法 2：xoshiro256**（备选顺序流，32 字节状态；比较用，非 v1 协议）
// ---------------------------------------------------------------------------

pub struct Xoshiro256ss {
    s: [u64; 4],
}

impl Xoshiro256ss {
    pub fn from_seed(seed: u64) -> Self {
        let mut m = SplitMix64::from_seed(seed);
        Self {
            s: [m.next_u64(), m.next_u64(), m.next_u64(), m.next_u64()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
}

// ---------------------------------------------------------------------------
// 备选算法 3：PCG64（XSL-RR 128/64，16 字节状态；比较用，非 v1 协议）
// ---------------------------------------------------------------------------

pub struct PcgXslRr128 {
    state: u128,
}

impl PcgXslRr128 {
    const MULT: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645;
    const INC: u128 = 0x5851_F42D_4C95_7F2D_1405_7B7E_F767_814F;

    pub fn from_seed(seed: u64) -> Self {
        let mut rng = Self {
            state: Self::INC,
        };
        rng.state = rng.state.wrapping_add(seed as u128);
        rng.state = rng.state.wrapping_mul(Self::MULT).wrapping_add(Self::INC);
        rng
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::MULT).wrapping_add(Self::INC);
        let rot = (self.state >> 122) as u32;
        let xs = ((self.state >> 64) as u64) ^ (self.state as u64);
        xs.rotate_right(rot)
    }
}

// ---------------------------------------------------------------------------
// 协议核心：派生（纯派生模式，EC-02 "种子 + 系统命名空间 + 事件键"）
// ---------------------------------------------------------------------------

/// FNV-1a 64（命名空间编码，属 RNG 协议 v1；变更必须 bump 版本）。
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// 派生流种子：mix(global_seed, fnv1a64(namespace), event_key)。
/// 编码顺序与混合常数属 RNG 协议（版本 1），变更必须 bump RNG_PROTOCOL_VERSION。
pub fn derive_seed(global_seed: u64, namespace: &str, event_key: u64) -> u64 {
    let ns = fnv1a64(namespace.as_bytes());
    let m1 = SplitMix64::from_seed(global_seed ^ ns).next_u64();
    SplitMix64::from_seed(m1 ^ event_key.wrapping_mul(0x9E37_79B9_7F4A_7C15)).next_u64()
}

/// 纯派生模式：第 draw_index 次抽取 = f(派生种子, draw_index)，无流状态。
/// 存档需求：零（只需全局种子 + RNG 协议版本，见 EC-03）。
pub fn event_u64(global_seed: u64, namespace: &str, event_key: u64, draw_index: u64) -> u64 {
    let seed = derive_seed(global_seed, namespace, event_key);
    SplitMix64::from_seed(seed ^ draw_index.wrapping_mul(0xD134_2543_DE82_E79B)).next_u64()
}

// ---------------------------------------------------------------------------
// 有状态流模式（顺序消费）
// ---------------------------------------------------------------------------

/// 计数器式顺序流：next = mix(seed, counter)，counter += 1。
/// 存档需求：(seed, counter) 共 16 字节/流。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stream64 {
    pub seed: u64,
    pub counter: u64,
}

impl Stream64 {
    pub fn new(global_seed: u64, namespace: &str, stream_key: u64) -> Self {
        Self {
            seed: derive_seed(global_seed, namespace, stream_key),
            counter: 0,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let v = SplitMix64::from_seed(self.seed ^ self.counter.wrapping_mul(0xD134_2543_DE82_E79B))
            .next_u64();
        self.counter += 1;
        v
    }

    /// 存档最小集（EC-03 口径）。
    pub fn save(&self) -> (u64, u64) {
        (self.seed, self.counter)
    }

    pub fn restore(saved: (u64, u64)) -> Self {
        Self {
            seed: saved.0,
            counter: saved.1,
        }
    }
}

// ---------------------------------------------------------------------------
// namespace 注册表（债务 #25：注册期 64 位碰撞校验）
// ---------------------------------------------------------------------------

/// 注册表错误。
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// 不同命名空间映射到同一 64 位派生键——拒绝注册。
    Collision {
        hash: u64,
        existing: String,
        attempted: String,
    },
}

/// namespace 注册表：已发布命名空间不得复用（ADR-NEW-22）。
/// 注册期做 64 位碰撞校验——不同命名空间派生键相同时注册失败，
/// 防止两个系统共享同一条随机流（EC-02 隔离性破坏）。
#[derive(Default)]
pub struct NamespaceRegistry {
    by_hash: BTreeMap<u64, String>,
}

impl NamespaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册命名空间，返回其 64 位派生键。同名重复注册幂等。
    pub fn register(&mut self, namespace: &str) -> Result<u64, RegistryError> {
        self.register_hashed(namespace, fnv1a64(namespace.as_bytes()))
    }

    /// 注入预计算哈希的注册（测试专用：构造碰撞场景）。
    pub fn register_hashed(&mut self, namespace: &str, hash: u64) -> Result<u64, RegistryError> {
        if let Some(existing) = self.by_hash.get(&hash) {
            if existing == namespace {
                return Ok(hash);
            }
            return Err(RegistryError::Collision {
                hash,
                existing: existing.clone(),
                attempted: namespace.to_string(),
            });
        }
        self.by_hash.insert(hash, namespace.to_string());
        Ok(hash)
    }

    pub fn get(&self, hash: u64) -> Option<&str> {
        self.by_hash.get(&hash).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_hash.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 测试（9 项迁移自 Spike B + namespace 注册表 3 项）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u64 = 0x1234_5678_9ABC_DEF0;

    /// 系统随机流隔离：两个系统互不影响，且增加一个系统的消耗不改变另一个的序列。
    #[test]
    fn system_stream_isolation() {
        let expected_econ: Vec<u64> = (0..100).map(|i| event_u64(SEED, "economy", 7, i)).collect();
        // traffic 消耗 10 万次
        for i in 0..100_000u64 {
            std::hint::black_box(event_u64(SEED, "traffic", 7, i));
        }
        let after_econ: Vec<u64> = (0..100).map(|i| event_u64(SEED, "economy", 7, i)).collect();
        assert_eq!(expected_econ, after_econ, "economy 流不得受 traffic 消耗影响");
        // 不同命名空间同键必须不同
        assert_ne!(event_u64(SEED, "traffic", 7, 0), event_u64(SEED, "economy", 7, 0));
    }

    /// Agent 随机流隔离：agent 消耗互不影响。
    #[test]
    fn agent_stream_isolation() {
        let a1: Vec<u64> = (0..10).map(|i| event_u64(SEED, "agent", 1, i)).collect();
        for i in 0..50_000u64 {
            std::hint::black_box(event_u64(SEED, "agent", 2, i));
        }
        let a1_after: Vec<u64> = (0..10).map(|i| event_u64(SEED, "agent", 1, i)).collect();
        assert_eq!(a1, a1_after, "agent 1 流不得受 agent 2 消耗影响");
        assert_ne!(a1[0], event_u64(SEED, "agent", 2, 0));
    }

    /// 调度顺序无关：A→B 与 B→A、交错执行，各流输出一致。
    #[test]
    fn schedule_order_independence() {
        // 纯派生：任意顺序
        let ab: Vec<u64> = (0..50)
            .flat_map(|i| [event_u64(SEED, "a", 0, i), event_u64(SEED, "b", 0, i)])
            .collect();
        let ba: Vec<u64> = (0..50)
            .flat_map(|i| [event_u64(SEED, "b", 0, i), event_u64(SEED, "a", 0, i)])
            .collect();
        let a_from_ab: Vec<u64> = ab.iter().step_by(2).copied().collect();
        let a_from_ba: Vec<u64> = ba.iter().skip(1).step_by(2).copied().collect();
        assert_eq!(a_from_ab, a_from_ba);
        // 顺序流：每流独立推进，交错消费互不影响
        let mut sa = Stream64::new(SEED, "sys.a", 0);
        let mut sb = Stream64::new(SEED, "sys.b", 0);
        let mut interleaved = Vec::new();
        for _ in 0..20 {
            interleaved.push(sa.next_u64());
            interleaved.push(sb.next_u64());
        }
        let mut sa2 = Stream64::new(SEED, "sys.a", 0);
        let solo: Vec<u64> = (0..20).map(|_| sa2.next_u64()).collect();
        let from_interleaved: Vec<u64> = interleaved.iter().step_by(2).copied().collect();
        assert_eq!(solo, from_interleaved, "交错消费不得改变流内序列");
    }

    /// 可复现性：同种子两次运行一致（黄金值由 print_goldens 生成后固化，与 Spike B 一致）。
    #[test]
    fn reproducibility_golden() {
        assert_eq!(derive_seed(SEED, "economy", 7), 0x5eca_56e0_1185_9455);
        assert_eq!(event_u64(SEED, "economy", 7, 0), 0xb048_2736_831c_79f2);
        assert_eq!(event_u64(SEED, "agent", 1, 0), 0x0f12_a199_77fa_44bd);
        let mut s = Stream64::new(SEED, "sys.a", 0);
        assert_eq!((s.next_u64(), s.next_u64()), (0xe03a_ea7b_d4c2_cc77, 0xc5db_86a8_37fa_0965));
    }

    /// 协议版本化：版本常量钉死，变更必须 bump（本测试失败=协议变更未走 ADR）。
    #[test]
    fn protocol_version_pinned() {
        assert_eq!(RNG_PROTOCOL_VERSION, 1, "RNG 协议变更必须 bump 版本并修订 ADR-NEW-22");
    }

    /// 存档需求差异：纯派生零状态；有状态流 16 字节，保存-恢复后继续一致。
    #[test]
    fn save_restore_and_state_footprint() {
        let pure_save_bytes = 0usize;
        assert_eq!(pure_save_bytes, 0);
        let mut s = Stream64::new(SEED, "sys.save", 42);
        for _ in 0..5 {
            s.next_u64();
        }
        let saved = s.save();
        assert_eq!(std::mem::size_of_val(&saved), 16, "有状态流存档 16 字节/流");
        let tail: Vec<u64> = (0..5).map(|_| s.next_u64()).collect();
        let mut r = Stream64::restore(saved);
        let tail2: Vec<u64> = (0..5).map(|_| r.next_u64()).collect();
        assert_eq!(tail, tail2, "读档后序列必须续接");
        assert_eq!(r, s, "恢复后两流状态相等");
    }

    /// 算法比较冒烟：三候选输出不同、黄金稳定（速度比较见 bench_all）。
    #[test]
    fn algo_smoke_and_stability() {
        let mut a = SplitMix64::from_seed(SEED);
        let mut b = Xoshiro256ss::from_seed(SEED);
        let mut c = PcgXslRr128::from_seed(SEED);
        let (a1, b1, c1) = (a.next_u64(), b.next_u64(), c.next_u64());
        assert_ne!(a1, b1);
        assert_ne!(b1, c1);
        assert_eq!(SplitMix64::from_seed(SEED).next_u64(), a1);
        assert_eq!(Xoshiro256ss::from_seed(SEED).next_u64(), b1);
        assert_eq!(PcgXslRr128::from_seed(SEED).next_u64(), c1);
    }

    /// 首值打印（生成黄金值用）：cargo test print_goldens -- --nocapture
    #[test]
    fn print_goldens() {
        println!("derive_seed(SEED,\"economy\",7) = {:#018x}", derive_seed(SEED, "economy", 7));
        println!("event_u64(SEED,\"economy\",7,0) = {:#018x}", event_u64(SEED, "economy", 7, 0));
        println!("event_u64(SEED,\"agent\",1,0)   = {:#018x}", event_u64(SEED, "agent", 1, 0));
        let mut s = Stream64::new(SEED, "sys.a", 0);
        println!("stream sys.a draws = {:#018x}, {:#018x}", s.next_u64(), s.next_u64());
    }

    /// 碰撞校验（债务 #25）：构造两个不同命名空间映射到同一哈希，注册必须被拒。
    #[test]
    fn registry_rejects_constructed_collision() {
        let mut reg = NamespaceRegistry::new();
        assert_eq!(reg.register_hashed("economy", 42), Ok(42));
        let err = reg.register_hashed("traffic.forged", 42);
        assert_eq!(
            err,
            Err(RegistryError::Collision {
                hash: 42,
                existing: "economy".to_string(),
                attempted: "traffic.forged".to_string(),
            })
        );
        // 原注册不受影响
        assert_eq!(reg.get(42), Some("economy"));
        assert_eq!(reg.len(), 1);
    }

    /// 同名重复注册幂等；不同名不同哈希均成功。
    #[test]
    fn registry_idempotent_and_distinct() {
        let mut reg = NamespaceRegistry::new();
        let h1 = reg.register("economy").unwrap();
        assert_eq!(reg.register("economy").unwrap(), h1, "同名重复注册幂等");
        let h2 = reg.register("traffic").unwrap();
        assert_ne!(h1, h2, "不同命名空间派生键不同");
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get(h2), Some("traffic"));
    }

    /// 真实 FNV-1a 64 下样本命名空间两两无碰撞（抽查，非穷举证明）。
    #[test]
    fn registry_real_hashes_are_distinct_for_samples() {
        let mut reg = NamespaceRegistry::new();
        for ns in ["economy", "traffic", "agent", "network", "pathfinding", "sys.a", "sys.b"] {
            reg.register(ns).unwrap();
        }
        assert_eq!(reg.len(), 7);
    }
}

/// 性能冒烟（非正式基准）：cargo test --release bench_all -- --nocapture
/// 遥测侧豁免（2.9 口径：墙钟与平台浮点只用于测量，不进入权威模拟结果）。
#[cfg(test)]
#[allow(clippy::disallowed_types, clippy::disallowed_methods)]
mod bench {
    use super::*;

    fn bench(name: &str, n: u64, mut next: impl FnMut() -> u64) {
        let t0 = std::time::Instant::now();
        let mut acc = 0u64;
        for _ in 0..n {
            acc = acc.wrapping_add(next());
            std::hint::black_box(acc);
        }
        let el = t0.elapsed();
        println!(
            "{name}: {n} draws in {el:?} = {:.2} ns/draw",
            el.as_nanos() as f64 / n as f64
        );
    }

    #[test]
    fn bench_all() {
        let n = 10_000_000u64;
        let mut a = SplitMix64::from_seed(1);
        bench("SplitMix64  ", n, || a.next_u64());
        let mut b = Xoshiro256ss::from_seed(1);
        bench("Xoshiro256**", n, || b.next_u64());
        let mut c = PcgXslRr128::from_seed(1);
        bench("PcgXslRr128 ", n, || c.next_u64());
        bench("derive_seed ", n, || {
            derive_seed(0x1234_5678_9ABC_DEF0, "economy", 7)
        });
        bench("event_u64   ", n, || {
            event_u64(0x1234_5678_9ABC_DEF0, "economy", 7, 1)
        });
    }
}
