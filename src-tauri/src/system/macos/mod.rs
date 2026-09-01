//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出 macOS 系统能力，按进程、进程树、launchd、控制者拆分。
//!
//! 主要职责：
//! - 转发进程结束与查询
//! - 转发 launchd 停止与控制者分析
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

mod controller;
mod launchd;
mod process;
mod process_tree;

pub use controller::detect_controller;
pub use launchd::stop_owning_service;
pub use process::{open_directory, process_priority, request_kill};
