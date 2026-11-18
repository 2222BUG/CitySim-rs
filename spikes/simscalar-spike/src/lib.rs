//! CitySim-rs Spike A: SimScalar candidate = Q32.32 fixed-point stored in an i64.
//!
//! STATUS: Proposed. Compiled and tested on rustc 1.100.0 (2026-11-18, 7/7 tests
//! passed). Note: the 2.1 version baseline states Rust 1.89 ("待复现验证");
//! pinning via rust-toolchain.toml or updating the baseline is pending
//! (see decision-log).
//!
//! Numeric contract (candidate, pending ADR):
//! - Representation: i64 raw value, 32 fractional bits. Range ±2^31, ulp 2^-32.
//! - Rounding: round to nearest, ties away from zero, for mul and div.
//! - Overflow: never implicit. `checked_*` returns Err, `saturating_*` clamps.
//!   There are deliberately no std::ops impls, so no hidden panic/wrap behaviour.
//! - Intermediates: i128, so results do not depend on platform word size.
//! - Time must NOT be stored as SimScalar seconds (overflows after ~68 years);
//!   keep an integer tick counter and derive dt from the tick rate.

#![forbid(unsafe_code)]

pub const FRAC_BITS: u32 = 32;
const ONE_RAW: i64 = 1 << FRAC_BITS;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SimScalar(i64);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarError {
    Overflow,
    DivByZero,
}

/// Round `x / 2^s` to nearest, ties away from zero. Requires 1 <= s <= 126.
fn round_shift(x: i128, s: u32) -> i128 {
    let half = 1i128 << (s - 1);
    if x >= 0 {
        (x + half) >> s
    } else {
        -((-x + half) >> s)
    }
}

/// Round `n / d` to nearest, ties away from zero. Requires d != 0.
fn round_div(n: i128, d: i128) -> i128 {
    let q = n / d;
    let r = n % d;
    if r.unsigned_abs() * 2 >= d.unsigned_abs() {
        if (n < 0) != (d < 0) {
            q - 1
        } else {
            q + 1
        }
    } else {
        q
    }
}

fn narrow(v: i128) -> Result<SimScalar, ScalarError> {
    i64::try_from(v)
        .map(SimScalar)
        .map_err(|_| ScalarError::Overflow)
}

impl SimScalar {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(ONE_RAW);
    pub const MAX: Self = Self(i64::MAX);
    pub const MIN: Self = Self(i64::MIN);

    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    pub const fn to_raw(self) -> i64 {
        self.0
    }

    /// Always exact: the i32 range equals the Q32.32 integer range.
    pub const fn from_int(n: i32) -> Self {
        Self((n as i64) << FRAC_BITS)
    }

    /// num/den rounded to nearest (ties away from zero). Use for constants like dt = 1/20.
    pub fn from_ratio(num: i64, den: i64) -> Result<Self, ScalarError> {
        if den == 0 {
            return Err(ScalarError::DivByZero);
        }
        narrow(round_div((num as i128) << FRAC_BITS, den as i128))
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, ScalarError> {
        self.0.checked_add(rhs.0).map(Self).ok_or(ScalarError::Overflow)
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, ScalarError> {
        self.0.checked_sub(rhs.0).map(Self).ok_or(ScalarError::Overflow)
    }

    pub fn checked_neg(self) -> Result<Self, ScalarError> {
        self.0.checked_neg().map(Self).ok_or(ScalarError::Overflow)
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self, ScalarError> {
        narrow(round_shift((self.0 as i128) * (rhs.0 as i128), FRAC_BITS))
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self, ScalarError> {
        if rhs.0 == 0 {
            return Err(ScalarError::DivByZero);
        }
        narrow(round_div((self.0 as i128) << FRAC_BITS, rhs.0 as i128))
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    pub fn saturating_mul(self, rhs: Self) -> Self {
        match self.checked_mul(rhs) {
            Ok(v) => v,
            Err(_) => {
                if (self.0 < 0) != (rhs.0 < 0) {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }
}

/// FNV-1a 64 over little-endian raw values. Spike-only digest; the real
/// GlobalDigest hash algorithm and versioning are decided separately.
pub fn fnv1a64(raws: &[i64]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for r in raws {
        for b in r.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(raw: i64) -> SimScalar {
        SimScalar::from_raw(raw)
    }

    struct XorShift64(u64);
    impl XorShift64 {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    #[test]
    fn constants_and_ratio() {
        assert_eq!(SimScalar::ONE.checked_mul(SimScalar::ONE), Ok(SimScalar::ONE));
        assert_eq!(SimScalar::from_ratio(1, 20).unwrap().to_raw(), 214_748_365);
        assert_eq!(SimScalar::from_ratio(1, 0), Err(ScalarError::DivByZero));
    }

    #[test]
    fn mul_ties_round_away_from_zero() {
        let half = s(1 << 31);
        assert_eq!(s(1).checked_mul(half), Ok(s(1)));
        assert_eq!(s(-1).checked_mul(half), Ok(s(-1)));
    }

    #[test]
    fn div_rounding_and_zero() {
        let third = SimScalar::ONE.checked_div(SimScalar::from_int(3)).unwrap();
        assert_eq!(third.to_raw(), 1_431_655_765);
        assert_eq!(SimScalar::ONE.checked_div(SimScalar::ZERO), Err(ScalarError::DivByZero));
    }

    #[test]
    fn overflow_is_explicit() {
        assert_eq!(SimScalar::MAX.checked_add(SimScalar::ONE), Err(ScalarError::Overflow));
        assert_eq!(SimScalar::MIN.checked_neg(), Err(ScalarError::Overflow));
        let big = SimScalar::from_int(i32::MAX);
        assert_eq!(big.checked_mul(SimScalar::from_int(2)), Err(ScalarError::Overflow));
        assert_eq!(SimScalar::MIN.checked_mul(SimScalar::from_int(-1)), Err(ScalarError::Overflow));
        assert_eq!(big.saturating_mul(big), SimScalar::MAX);
        assert_eq!(big.saturating_mul(SimScalar::from_int(i32::MIN)), SimScalar::MIN);
    }

    #[test]
    fn mul_properties_over_random_inputs() {
        let mut rng = XorShift64(0x9E37_79B9_7F4A_7C15);
        for _ in 0..100_000 {
            // 46-bit signed raw values: products always fit after the shift.
            let a = s((rng.next() as i64) >> 18);
            let b = s((rng.next() as i64) >> 18);
            let ab = a.checked_mul(b).unwrap();
            assert_eq!(ab, b.checked_mul(a).unwrap(), "commutative");
            assert_eq!(a.checked_mul(SimScalar::ONE).unwrap(), a, "identity");
            let na = a.checked_neg().unwrap();
            assert_eq!(na.checked_mul(b).unwrap(), ab.checked_neg().unwrap(), "sign symmetry");
        }
    }

    /// 1e6 ticks at 20 TPS, v = 15 m/s. Python emulation predicts +3_000_000 raw (~0.7 mm).
    #[test]
    fn drift_golden_20tps() {
        let dt = SimScalar::from_ratio(1, 20).unwrap();
        let v = SimScalar::from_int(15);
        let step = v.checked_mul(dt).unwrap();
        let mut pos = SimScalar::ZERO;
        for _ in 0..1_000_000 {
            pos = pos.checked_add(step).unwrap();
        }
        assert_eq!(pos.to_raw(), 3_221_225_475_000_000);
        let exact = 750_000i64 << FRAC_BITS;
        assert!((pos.to_raw() - exact).abs() < ONE_RAW / 1000, "drift must stay under 1 mm");
    }

    #[test]
    fn digest_is_stable() {
        assert_eq!(fnv1a64(&[1, -1, i64::MAX]), 0x99fc_c379_ecfc_4214);
    }
}

/// Step 2（2026-11-18）：2.10 定案参数（tick = 30 Hz）下的正式核对测试。
/// 输入：项目负责人定案参数（Cities: Skylines 2 参考值）。
/// 配套：docs/benchmarks/simscalar-formal-check.py（Python 侧全量核对）。
#[cfg(test)]
mod step2_tests {
    use super::*;

    /// dt = 1/30 s 一次派生（from_ratio），黄金值 round(2^32 / 30) = 143_165_577，
    /// 舍入误差 +1.087e-10 s 固定，写入黄金测试（ADR-NEW-18 数值契约 2）。
    #[test]
    fn dt_30hz_from_ratio_golden() {
        let dt = SimScalar::from_ratio(1, 30).unwrap();
        assert_eq!(dt.to_raw(), 143_165_577);
    }

    /// v = 15 m/s 的每 tick 步长黄金值：step_raw = round_shift(v_raw * dt_raw, 32) = 2_147_483_655
    /// （0.5 m + 7 raw，步长误差固定 +1.63e-9 m）。1e6 ticks 常速积分漂移 +7e6 raw ≈ +1.629 mm。
    #[test]
    fn drift_golden_30tps() {
        let dt = SimScalar::from_ratio(1, 30).unwrap();
        let v = SimScalar::from_int(15);
        let step = v.checked_mul(dt).unwrap();
        assert_eq!(step.to_raw(), 2_147_483_655);
        let total = 1_000_000i64 * step.to_raw();
        let ideal = 500_000i64 * (1i64 << 32); // 1e6 ticks × 0.5 m = 500 km = 500_000 m，精确表示
        assert_eq!(total - ideal, 7_000_000, "drift must be exactly +7 raw per tick");
    }

    /// 中间乘法位宽：50 m/s × dt 需 66 bit，i128 中间值足够（formal-check [4]）。
    #[test]
    fn intermediate_width_fits_i128() {
        let v_raw: i128 = 50 * (1i128 << 32);
        let dt_raw: i128 = 143_165_577;
        let prod = v_raw * dt_raw;
        assert!(prod > 0 && prod < (1i128 << 126), "50 m/s * dt must fit i128 with margin");
    }

    /// 16 km 地图：半幅 8000 m 远小于 ±2^31 m；1 mm 分辨率可表示（LSB = 2^-32 m）。
    #[test]
    fn map_16km_and_1mm_fit() {
        let half_map = SimScalar::from_int(8000);
        assert_eq!(half_map.to_raw(), 8000i64 * (1i64 << 32));
        let mm = SimScalar::from_ratio(1, 1000).unwrap();
        assert_eq!(mm.to_raw(), 4_294_967); // round(2^32 / 1000)，1 mm 远大于 LSB
    }

    /// 性能冒烟（非正式基准，无阈值断言）：1e7 次 saturating_mul 计时并打印，
    /// 运行方式 cargo test --release perf_smoke_mul -- --nocapture。
    #[test]
    // 遥测侧豁免（2.9 口径）：墙钟与 f64 仅用于测量输出，不进入权威模拟结果。
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    fn perf_smoke_mul() {
        let m = SimScalar::from_ratio(3, 2).unwrap();
        let mut acc = SimScalar::from_int(1);
        let n = 10_000_000u64;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            acc = acc.saturating_mul(m);
            std::hint::black_box(acc);
        }
        let el = t0.elapsed();
        println!(
            "perf_smoke_mul: {} ops in {:?} = {:.1} ns/op (非正式基准，机器相关)",
            n,
            el,
            el.as_nanos() as f64 / n as f64
        );
    }
}
