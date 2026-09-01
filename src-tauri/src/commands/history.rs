//! 文件名称：history.rs
//!
//! 文件功能：
//! 历史记录相关 Tauri 命令。
//!
//! 主要职责：
//! - 列出历史
//! - 清空历史
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::State;

use crate::errors::AppError;
use crate::models::HistoryRecord;
use crate::services::history_service;
use crate::state::AppState;

/// 列出操作历史。
#[tauri::command]
pub fn list_history(state: State<'_, AppState>) -> Result<Vec<HistoryRecord>, AppError> {
    history_service::list(&state.data_dir)
}

/// 清空操作历史。
#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), AppError> {
    history_service::clear(&state.data_dir)
}
