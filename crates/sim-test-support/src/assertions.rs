//! 模拟状态断言（EC-07 assertions）。

/// 断言：摘要一致（确定性回归 / 读档等价通用）。
#[track_caller]
pub fn assert_digest_eq(context: &str, d1: u64, d2: u64) {
    assert_eq!(d1, d2, "digest mismatch in {}: {:#018x} != {:#018x}", context, d1, d2);
}

/// 断言：序列按稳定键升序（2.9 条 4——并行提交/查询结果按稳定键排序）。
#[track_caller]
pub fn assert_sorted_by_key(keys: &[u64]) {
    for w in keys.windows(2) {
        assert!(w[0] <= w[1], "keys not in stable order: {} > {}", w[0], w[1]);
    }
}

/// 断言：读档后高水位不得回退（ID 不复用，ADR-NEW-20 / EC-03 身份类）。
#[track_caller]
pub fn assert_high_water_non_decreasing(context: &str, before: u64, after: u64) {
    assert!(after >= before, "high water regressed in {}: {} -> {}", context, before, after);
}

/// 断言：tick 边界语义（存档/预算调整/命令生效仅发生在 tick 边界）。
#[track_caller]
pub fn assert_tick_boundary(context: &str, current_tick: u64, boundary_tick: u64) {
    assert_eq!(
        current_tick, boundary_tick,
        "{} must occur at a tick boundary: current {} != boundary {}",
        context, current_tick, boundary_tick
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertions_pass_on_valid_state() {
        assert_digest_eq("mock", 1, 1);
        assert_sorted_by_key(&[1, 1, 2, 3]);
        assert_high_water_non_decreasing("alloc", 5, 5);
        assert_high_water_non_decreasing("alloc", 5, 6);
        assert_tick_boundary("save", 40, 40);
    }

    #[test]
    #[should_panic(expected = "digest mismatch")]
    fn digest_mismatch_panics() {
        assert_digest_eq("mock", 1, 2);
    }
}
