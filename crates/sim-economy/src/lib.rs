//! 经济及城市系统：承载后续城市模拟能力（主文档 2.2 架构分层）。
//! 权威crate。
//!
//! Stage 1 T1 骨架（2026-11-19 建立）——任务清单见 docs/architecture/Stage-1-Task-List.md。
//! 依赖方向约束见 docs/architecture/crate-authority-list.md 与 .github/scripts/dep_graph_check.py。

/// 所属架构层（依赖方向检查与文档用）。
pub const LAYER: &str = "经济及城市系统";
/// 是否权威 crate（2.9 确定性边界内；权威 crate 置严格 clippy.toml）。
pub const AUTHORITATIVE: bool = true;

/// 供调用方与测试以函数调用读取权威标记（clippy 允许的运行时读取点）。
pub fn authoritative() -> bool {
    AUTHORITATIVE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_is_pinned() {
        assert!(authoritative());
        assert!(!LAYER.is_empty());
    }
}
