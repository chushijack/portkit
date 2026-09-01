//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出应用常量配置。
//!
//! 主要职责：
//! - 暴露常用端口与受保护进程名单
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod constants;

pub use constants::{is_common_port, COMMON_PORT_ITEMS, PROTECTED_PROCESS_NAMES};
