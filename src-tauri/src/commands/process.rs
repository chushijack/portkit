//! 文件名称：process.rs
//!
//! 文件功能：
//! 进程查询与结束相关 Tauri 命令。
//!
//! 主要职责：
//! - 查询进程详情
//! - 打开进程目录
//! - 按指定方式结束进程并写入历史
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::State;

use crate::errors::AppError;
use crate::models::{KillMode, KillOutcome, ProcessDetail};
use crate::services::{history_service, process_service};
use crate::state::AppState;

/// 查询进程详情。
#[tauri::command]
pub async fn get_process_detail(pid: u32) -> Result<ProcessDetail, AppError> {
    tauri::async_runtime::spawn_blocking(move || process_service::get_process_detail(pid))
        .await
        .map_err(|error| AppError::SystemError(error.to_string()))?
}

/// 打开进程所在目录。
#[tauri::command]
pub async fn open_process_directory(pid: u32) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || process_service::open_process_directory(pid))
        .await
        .map_err(|error| AppError::SystemError(error.to_string()))?
}

/// 按指定方式结束进程并记录关闭端口历史。被重新拉起时不记成功历史。
#[tauri::command]
pub async fn kill_process(
    state: State<'_, AppState>,
    pid: u32,
    port: Option<u16>,
    protocol: Option<String>,
    process_name: Option<String>,
    mode: Option<KillMode>,
) -> Result<KillOutcome, AppError> {
    let data_dir = state.data_dir.clone();
    let history_name = process_name.clone();
    let mode = mode.unwrap_or_default();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        process_service::kill_process(pid, port, process_name, mode)
    })
    .await
    .map_err(|error| AppError::SystemError(error.to_string()))??;
    if matches!(outcome, KillOutcome::Released { .. }) {
        history_service::record_kill(&data_dir, port, protocol, history_name, pid)?;
    }
    Ok(outcome)
}
