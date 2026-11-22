//! CitySim-rs sim-codec（Stage 1 T6）：canonical 序列化编码器（ADR-NEW-19）。
//!
//! 存档与 GlobalDigest 共用**底层编码规则**（字段编码、端序、可选值、字段顺序、
//! 稳定身份编码、版本标识），但**顶层 Schema 相互独立**（存档/摘要各自版本化、
//! 迁移逻辑独立）——P0-04 修订口径。
//!
//! 编码规则 v1：
//! - 整数一律小端定长（u8/u16/u32/u64/i64）；usize 以 u64 编码。
//! - 变长数据 = u64 长度前缀 + 原始字节；字符串 = UTF-8 字节同上。
//! - 浮点禁入（2.9 条 3；权威侧无 f32/f64）。
//! - domain separation：编码流首部含编码规则版本、模块标识、类型标识
//!   （ADR-NEW-19 条 2）。
//! - 字段顺序即编码顺序（调用方约定，由各 Schema 文档固定）。
#![forbid(unsafe_code)]

/// 编码规则版本标识（变更必须 bump，并同步 digest_version/存档 Schema 策略）。
pub const CODEC_VERSION: &str = "CSIMCOD1";

/// canonical 编码器：小端、确定性、带 domain separation 头。
pub struct CanonicalWriter {
    buf: Vec<u8>,
}

impl CanonicalWriter {
    /// 开始一个带域分隔的编码流：CSIMCOD1 | module | kind | …字段…
    pub fn new(module: &str, kind: &str) -> Self {
        let mut w = Self { buf: Vec::new() };
        w.str(CODEC_VERSION);
        w.str(module);
        w.str(kind);
        w
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// usize 以 u64 编码（平台字长无关，2.9 口径）。
    pub fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }

    /// 变长字节：u64 长度前缀 + 原始字节。
    pub fn bytes(&mut self, b: &[u8]) {
        self.u64(b.len() as u64);
        self.buf.extend_from_slice(b);
    }

    /// 字符串：UTF-8 字节 + u64 长度前缀。
    pub fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }

    /// 序列前缀：u64 元素数 + 各元素由调用方逐个写入。
    pub fn seq_len(&mut self, len: usize) {
        self.usize(len);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 字节布局由规范直接推导（小端），断言精确字节。
    #[test]
    fn integer_layout_is_little_endian_fixed_width() {
        let mut w = CanonicalWriter::new("t", "x");
        w.u8(1);
        w.bool(true);
        w.u16(0x0203);
        w.u32(0x0405_0607);
        w.u64(0x0809_0a0b_0c0d_0e0f);
        w.i64(-1);
        assert_eq!(
            w.as_bytes()[w.as_bytes().len() - 24..],
            [
                1u8,
                1u8,
                0x03, 0x02,
                0x07, 0x06, 0x05, 0x04,
                0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ][..]
        );
    }

    /// 变长前缀与域分隔头。
    #[test]
    fn domain_header_and_varlen_prefix() {
        let mut w = CanonicalWriter::new("sim-agent", "AgentState");
        w.str("ab");
        let b = w.into_bytes();
        // 头：CODEC_VERSION | "sim-agent" | "AgentState"，各带 u64 长度前缀
        assert!(b.starts_with(&[8, 0, 0, 0, 0, 0, 0, 0]), "CODEC_VERSION 长度前缀=8");
        assert!(b.windows(8).any(|s| s == b"CSIMCOD1"));
        assert!(b.windows(9).any(|s| s == b"sim-agent"));
        assert!(b.windows(10).any(|s| s == b"AgentState"));
        // 尾部："ab" = 长度 2 + 'a' 'b'
        assert_eq!(&b[b.len() - 10..], &[2, 0, 0, 0, 0, 0, 0, 0, b'a', b'b']);
    }

    /// 同输入两次编码逐字节一致（确定性）。
    #[test]
    fn encoding_is_deterministic() {
        let f = || {
            let mut w = CanonicalWriter::new("m", "k");
            w.u64(42);
            w.str("稳");
            w.seq_len(2);
            w.i64(-7);
            w.i64(7);
            w.into_bytes()
        };
        assert_eq!(f(), f());
    }

    /// 浮点禁入由类型系统保证：编码器不提供 f32/f64 方法（本测试为契约记录）。
    #[test]
    fn no_float_entry_points() {
        let mut w = CanonicalWriter::new("m", "k");
        w.u32(0x3f80_0000); // 若需记录位模式，以整数显式写入
        // 头 34 字节（CSIMCOD1/m/k 各 8 字节长度前缀）+ u32 4 字节
        assert_eq!(w.as_bytes().len(), 38);
    }

    /// canonical 字节级金标准（审计3 P2-b）：编码布局以手算期望字节钉死——
    /// 信封=CODEC_VERSION/module/kind 各 u64 长度前缀+UTF-8；载荷小端定长。
    /// 任何布局改动必须显式更新本黄金（=编码变更须 bump digest_version）。
    #[test]
    fn canonical_bytes_golden() {
        let mut w = CanonicalWriter::new("m", "k");
        w.u8(0xAB);
        w.u64(0x0102_0304_0506_0708);
        w.str("hi");
        let expected: Vec<u8> = [
            // 信封 34B：u64(8) "CSIMCOD1" | u64(1) "m" | u64(1) "k"
            &[8, 0, 0, 0, 0, 0, 0, 0],
            b"CSIMCOD1".as_slice(),
            &[1, 0, 0, 0, 0, 0, 0, 0],
            b"m",
            &[1, 0, 0, 0, 0, 0, 0, 0],
            b"k",
            // 载荷 19B：u8 0xAB | u64 小端 | u64(2) "hi"
            &[0xAB],
            &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
            &[2, 0, 0, 0, 0, 0, 0, 0],
            b"hi",
        ]
        .concat();
        assert_eq!(
            w.as_bytes(),
            expected.as_slice(),
            "canonical 布局黄金不匹配（编码变更须 bump digest_version）"
        );
        assert_eq!(w.as_bytes().len(), 53);
    }
}
