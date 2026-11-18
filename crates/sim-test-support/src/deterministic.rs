//! 确定性黄金测试 + 读档等价测试模板（EC-07 deterministic）。
//!
//! 模板来源：docs/benchmarks/SaveRestore-Validation-REPORT.md（Step 5 已实证
//! 强/弱两种形式）。任何实现 [`SimHarness`] 的模拟核心都可直接套用。
//!
//! 确定性测试口径（EC-07）：固定输入和种子，重复运行并比较摘要；
//! 覆盖线程数无关性（R3 每日构建）与读档等价性（R2 门禁，强形式）。

/// 模拟器测试接口。
pub trait SimHarness {
    /// 运行到指定 tick，返回该 tick 的状态摘要。
    fn run_to(&mut self, tick: u64) -> u64;
    /// 存档（仅 tick 边界语义由实现保证，EC-03）。
    fn save(&self) -> Vec<u8>;
    /// 读档。
    fn load(&mut self, blob: &[u8]);
}

/// 强读档等价（R2 强形式，EC-11）：
/// - run_a：直接跑到 M，取摘要 d_a；
/// - run_b：跑到 N（N < M）并存档；
/// - run_c：新实例载入同一存档，跑到 M，取摘要 d_c；
/// - 断言由调用方完成：d_a == d_c。
///
/// 依赖在途任务"保存并恢复"语义（ADR-NEW-16 定案）；若政策改为
/// "丢弃并重新入队"，必须改用 [`weak_form_digests`] 并另行 ADR 修订。
pub fn strong_readback_digests<H: SimHarness>(
    make: impl Fn() -> H,
    n: u64,
    m: u64,
) -> (u64, u64) {
    assert!(n < m, "save point N must precede target tick M");
    let mut a = make();
    let d_a = a.run_to(m);
    let mut b = make();
    b.run_to(n);
    let blob = b.save();
    let mut c = make();
    c.load(&blob);
    let d_c = c.run_to(m);
    (d_a, d_c)
}

/// 弱形式等价（政策改"丢弃并重新入队"时的降级口径）：
/// 两次独立的 save/load 运行之间摘要一致。
pub fn weak_form_digests<H: SimHarness>(make: impl Fn() -> H, n: u64, m: u64) -> (u64, u64) {
    assert!(n < m, "save point N must precede target tick M");
    let run = |fresh: bool| -> u64 {
        let mut b = make();
        b.run_to(n);
        let blob = b.save();
        let mut c = make();
        c.load(&blob);
        let _ = fresh;
        c.run_to(m)
    };
    (run(true), run(false))
}

/// 黄金值断言辅助：固定输入重复运行两次，摘要必须一致。
pub fn repeat_run_digests<H: SimHarness>(make: impl Fn() -> H, m: u64) -> (u64, u64) {
    let mut a = make();
    let d1 = a.run_to(m);
    let mut b = make();
    let d2 = b.run_to(m);
    (d1, d2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 玩具计数器模拟器：演示模板自检（非真实模拟核心）。
    struct CounterSim {
        tick: u64,
        acc: u64,
    }

    const MULT: u64 = 6_364_136_223_846_793_005;
    const INC: u64 = 1_442_695_040_888_963_407;

    impl CounterSim {
        fn new() -> CounterSim {
            CounterSim { tick: 0, acc: 0x5EED }
        }
    }

    impl SimHarness for CounterSim {
        fn run_to(&mut self, tick: u64) -> u64 {
            while self.tick < tick {
                self.tick += 1;
                self.acc = self.acc.wrapping_mul(MULT).wrapping_add(INC ^ self.tick);
            }
            self.acc ^ self.tick
        }
        fn save(&self) -> Vec<u8> {
            let mut v = Vec::new();
            v.extend_from_slice(&self.tick.to_le_bytes());
            v.extend_from_slice(&self.acc.to_le_bytes());
            v
        }
        fn load(&mut self, blob: &[u8]) {
            assert_eq!(blob.len(), 16, "toy save must be 16 bytes");
            self.tick = u64::from_le_bytes(blob[0..8].try_into().unwrap());
            self.acc = u64::from_le_bytes(blob[8..16].try_into().unwrap());
        }
    }

    #[test]
    fn strong_readback_template_works_on_mock() {
        let (d_a, d_c) = strong_readback_digests(CounterSim::new, 40, 100);
        assert_eq!(d_a, d_c, "strong readback equivalence must hold on mock");
    }

    #[test]
    fn weak_form_template_works_on_mock() {
        let (d1, d2) = weak_form_digests(CounterSim::new, 40, 100);
        assert_eq!(d1, d2);
    }

    #[test]
    fn repeat_run_is_deterministic_on_mock() {
        let (d1, d2) = repeat_run_digests(CounterSim::new, 77);
        assert_eq!(d1, d2);
    }
}
