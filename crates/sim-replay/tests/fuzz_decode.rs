//! 解码拒绝路径鲁棒性（开源采纳 O3，fuzz-lite）：任意字节输入只许 Err/Ok，绝不 panic。
//! 失败输入落 `proptest-regressions/` 回放；cargo-fuzz 布局见 fuzz/（nightly 专项）。
use proptest::prelude::*;
use sim_replay::ReplayInput;

proptest! {
    #[test]
    fn decode_never_panics_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let _ = ReplayInput::decode(&bytes); // 唯一断言=不 panic（Ok/Err 皆合法）
    }

    /// 截断/扩展扰动同样只许 Err/Ok。
    #[test]
    fn decode_never_panics_on_mutated_header(
        pad in proptest::collection::vec(any::<u8>(), 0..64),
        cut in 0usize..64,
    ) {
        let mut bytes = b"CSIMCOD1".to_vec();
        bytes.extend_from_slice(&pad);
        bytes.truncate(bytes.len().saturating_sub(cut));
        let _ = ReplayInput::decode(&bytes);
    }
}
