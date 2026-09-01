//! 文件名称：unsupported.rs
//!
//! 文件功能：
//! 非 Windows / macOS 平台的系统能力占位实现。
//!
//! 主要职责：
//! - 明确返回不支持错误，避免误编译成空操作
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use crate::errors::AppError;
use crate::models::{KillMode, ProcessController, ProcessIdentity};

/// 当前平台不支持结束进程。
pub fn request_kill(_pid: u32, _mode: KillMode) {}

/// 当前平台不支持查询进程是否仍在运行。
pub fn process_is_running(_pid: u32) -> bool {
    false
}

/// 当前平台不支持查询优先级。
pub fn process_priority(_pid: u32) -> String {
    "normal".to_string()
}

/// 当前平台不支持打开目录。
pub fn open_directory(_path: &str) -> Result<(), AppError> {
    Err(AppError::UnsupportedPlatform)
}

/// 当前平台不支持按 PID 查询服务名。
pub fn service_name_by_pid(_pid: u32) -> Option<String> {
    None
}

/// 当前平台不支持按 PID 停止服务。
pub fn stop_owning_service(_pid: u32) -> Result<bool, AppError> {
    Err(AppError::UnsupportedPlatform)
}

/// 当前平台无法分析进程控制者。
pub fn detect_controller(_process: &ProcessIdentity) -> ProcessController {
    ProcessController::Unknown
}

