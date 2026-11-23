//! cargo-fuzz 目标：sim-save 存档解码拒绝路径（任意字节只许 Err/Ok，不 panic）。
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = sim_save::SimState::load(data);
});
