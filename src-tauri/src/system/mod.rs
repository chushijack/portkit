//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出操作系统能力，按平台选择实现。
//!
//! 主要职责：
//! - 暴露端口套接字枚举
//! - 暴露进程查询、结束与打开目录
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod process;
pub mod socket;

#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(windows, target_os = "macos")))]
mod unsupported;

pub use process::{
    parent_controller, pids_named_under, process_identity, query_process, query_processes,
    same_named_ancestors, ProcessSnapshot,
};
pub use socket::{list_sockets, SocketRecord};

#[cfg(windows)]
pub use windows::{
    detect_controller, is_portable_installation, open_directory, process_priority, request_kill,
    stop_owning_service,
};

#[cfg(target_os = "macos")]
pub use macos::{
    detect_controller, open_directory, process_priority, request_kill, stop_owning_service,
};

#[cfg(not(any(windows, target_os = "macos")))]
pub use unsupported::{
    detect_controller, open_directory, process_priority, request_kill, stop_owning_service,
};
