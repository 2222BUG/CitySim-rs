//! sim-test-support —— EC-07 统一测试基础设施（Stage 0 Step 6）
//!
//! 组织（EC-07）：
//! - `fixtures`：测试数据和场景。
//! - `deterministic`：确定性黄金测试（含读档等价测试模板）。
//! - `property`：属性测试不变量助手。
//! - `benchmark`：基准测试模板。
//! - `assertions`：模拟状态断言。
//! - `chaos`：确定性注入钩子（R3 线程数无关性测试：忙等延迟 + 种子化乱序）。
//!
//! 约束（EC-07）：测试支持模块本身不应成为核心运行时依赖——本 crate 只允许
//! 作为 dev-dependency 引入权威模拟 crate。
//!
//! 口径：本 crate 属测试/遥测侧，允许使用墙钟与平台整数；权威模拟系统
//! 仍受 2.9 确定性边界约束（禁墙钟、禁浮点、禁迭代顺序依赖）。
#![forbid(unsafe_code)]

pub mod assertions;
pub mod benchmark;
pub mod chaos;
pub mod deterministic;
pub mod fixtures;
pub mod property;
