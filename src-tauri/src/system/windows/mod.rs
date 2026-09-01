//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出 Windows 系统能力，按进程、进程树、服务、控制者拆分。
//!
//! 主要职责：
//! - 转发进程结束与查询
//! - 转发服务停止与控制者分析
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

mod controller;
mod installation;
mod process;
mod process_tree;
mod service;

pub use controller::detect_controller;
pub use installation::is_portable_installation;
pub use process::{open_directory, process_priority, request_kill};
pub use service::stop_owning_service;

pub(super) use process::{process_account_name, process_start_unix};
