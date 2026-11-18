//! 确定性注入（EC-11 R3 线程数无关性测试的注入钩子预留）。
//!
//! 口径（eng-audit 裁定）："注入延迟"必须确定性——用忙等次数替代 sleep，
//! 乱序以任务 ID + chaos_seed 派生的键驱动，禁止墙钟/线程 sleep，
//! 否则测试自身违反 2.9 确定性边界。
//!
//! 用法：调度器/任务执行器在执行任务前调用 [`chaos_spin`]，在收集提交
//! 结果处用 [`chaos_sort_key`] 对提交顺序做种子化扰动；chaos_seed = 0 时
//! 无扰动（正常路径）。

/// 确定性忙等次数（替代 sleep 的注入延迟）。
///
/// 由 (task_id, chaos_seed) 派生；同输入必同输出（可复现）。
/// 返回值为忙等迭代次数，调用方在非权威测试路径执行。
pub fn chaos_spin(task_id: u64, chaos_seed: u64) -> u64 {
    if chaos_seed == 0 {
        return 0;
    }
    let mut x = task_id ^ chaos_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    // splitmix64 finalizer：打散低位相关性
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x % 1_024
}

/// 确定性乱序键：按 (chaos_sort_key(task_id, seed), task_id) 排序即得
/// 种子化扰动的稳定顺序（同 seed 同顺序，跨线程可复现）。
pub fn chaos_sort_key(task_id: u64, chaos_seed: u64) -> u64 {
    if chaos_seed == 0 {
        return task_id; // 正常路径：按稳定键原序
    }
    chaos_spin(task_id, chaos_seed)
        .wrapping_mul(0x0001_0000_0001_F11D)
        .wrapping_add(task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaos_is_deterministic_and_seed_zero_is_noop() {
        assert_eq!(chaos_spin(42, 7), chaos_spin(42, 7), "same input, same delay");
        assert_eq!(chaos_spin(42, 0), 0, "seed 0 must disable injection");
        assert_eq!(chaos_sort_key(42, 0), 42);
    }

    #[test]
    fn chaos_keys_produce_distinct_order() {
        let mut ids: Vec<u64> = (0..64).collect();
        let mut keyed: Vec<(u64, u64)> = ids.iter().map(|&i| (chaos_sort_key(i, 123), i)).collect();
        keyed.sort_unstable();
        let shuffled: Vec<u64> = keyed.into_iter().map(|(_, i)| i).collect();
        assert_ne!(ids, shuffled, "chaos seed should perturb order");
        // 稳定性：再排一次同结果
        let mut again: Vec<(u64, u64)> = ids.iter().map(|&i| (chaos_sort_key(i, 123), i)).collect();
        again.sort_unstable();
        let shuffled2: Vec<u64> = again.into_iter().map(|(_, i)| i).collect();
        assert_eq!(shuffled, shuffled2);
        ids.clear();
    }
}
