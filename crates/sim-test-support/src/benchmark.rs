//! 基准测试模板（EC-07 benchmark）：固定场景、硬件、构建模式和 Tick 数，
//! 记录运行耗时与内存占用。
//!
//! 口径：本模块属测试/遥测侧，允许墙钟（2.9 只约束权威模拟系统）；
//! 正式基准记录统一存 docs/benchmarks/（证据纪律）。

/// 一次基准运行的记录。
#[derive(Clone, Debug, PartialEq)]
pub struct BenchRecord {
    pub scenario: String,
    pub hardware_note: String,
    pub build_mode: String,
    pub ticks: u64,
    pub wall_ms: f64,
    pub memory_note: String,
}

impl BenchRecord {
    /// 汇报行（供 docs/benchmarks/ 归档）。
    pub fn to_report_line(&self) -> String {
        format!(
            "{} | {} | {} | {} ticks | {:.3} ms | {}",
            self.scenario, self.hardware_note, self.build_mode, self.ticks, self.wall_ms,
            self.memory_note
        )
    }
}

/// 构造一条基准记录。
pub fn record(
    scenario: &str,
    hardware_note: &str,
    build_mode: &str,
    ticks: u64,
    wall_ms: f64,
    memory_note: &str,
) -> BenchRecord {
    BenchRecord {
        scenario: scenario.to_string(),
        hardware_note: hardware_note.to_string(),
        build_mode: build_mode.to_string(),
        ticks,
        wall_ms,
        memory_note: memory_note.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_line_contains_required_fields() {
        let r = record("r2-gate-10k", "win-x86_64 rustc1.100", "release", 1_000, 1234.5, "n/a");
        let line = r.to_report_line();
        assert!(line.contains("r2-gate-10k"));
        assert!(line.contains("release"));
        assert!(line.contains("1000 ticks"));
        assert!(line.contains("1234.500 ms"));
    }
}
