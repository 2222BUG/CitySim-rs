//! 属性测试不变量助手（EC-07 property）。
//!
//! 对应 EC-07 不变量清单：
//! 1. 路网边引用合法；
//! 2. Agent 身份不重复；
//! 3. 事务发布前后状态有效；
//! 4. 数值运算遵守预期边界；
//! 5. 过期寻路结果的重算次数不超过上限。
//!
//! 本模块只提供不变量判定函数；随机输入由调用方以固定种子生成
//! （确定性属性测试，2.9 口径）。

/// 不变量 1：路网边引用合法（端点 < node_count，cost 为正）。
pub fn edges_reference_valid_nodes(edges: &[(u64, u64, u64)], node_count: u64) -> bool {
    edges.iter().all(|&(a, b, cost)| a < node_count && b < node_count && a != b && cost > 0)
}

/// 不变量 2：身份不重复（AgentId / 请求 ID / 事务 ID 通用）。
pub fn ids_unique(ids: &[u64]) -> bool {
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    sorted.windows(2).all(|w| w[0] != w[1])
}

/// 不变量 3：事务发布前后状态有效——版本严格单调递增且只前进一格
/// （原子发布，ADR-NEW-17；失败不得改版本，Spike C 已实证）。
pub fn tx_publish_valid(before_version: u64, after_version: u64, publish_ok: bool) -> bool {
    if publish_ok {
        after_version == before_version + 1
    } else {
        after_version == before_version
    }
}

/// 不变量 4：数值运算遵守预期边界（SimScalar / 中间值位宽，ADR-NEW-21）。
pub fn within_bounds(value: i128, min: i128, max: i128) -> bool {
    value >= min && value <= max
}

/// 不变量 5：过期寻路结果的重算次数不超过上限（ADR-NEW-17 / ADR-NEW-23：
/// recompute_limit，超限进入兜底而非继续重算）。
pub fn recompute_within_limit(count: u64, limit: u64) -> bool {
    count <= limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariants_hold_on_valid_inputs() {
        assert!(edges_reference_valid_nodes(&[(0, 1, 100), (1, 2, 100)], 3));
        assert!(!edges_reference_valid_nodes(&[(0, 1, 100), (1, 9, 100)], 3), "dangling node");
        assert!(!edges_reference_valid_nodes(&[(0, 1, 0)], 2), "zero cost");

        assert!(ids_unique(&[3, 1, 2]));
        assert!(!ids_unique(&[1, 1]));

        assert!(tx_publish_valid(7, 8, true));
        assert!(!tx_publish_valid(7, 9, true), "version must advance by one");
        assert!(tx_publish_valid(7, 7, false), "failed tx must not change version");

        assert!(within_bounds(0, i64::MIN as i128, i64::MAX as i128));
        assert!(!within_bounds(i64::MAX as i128 + 1, i64::MIN as i128, i64::MAX as i128));

        assert!(recompute_within_limit(3, 3));
        assert!(!recompute_within_limit(4, 3));
    }
}
