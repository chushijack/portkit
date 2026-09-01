//! 文件名称：favorite.rs
//!
//! 文件功能：
//! 收藏端口相关 Tauri 命令。
//!
//! 主要职责：
//! - 列出、新增、更新备注、删除收藏
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::State;

use crate::errors::AppError;
use crate::models::FavoritePort;
use crate::services::favorite_service;
use crate::state::AppState;

/// 列出收藏端口。
#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> Result<Vec<FavoritePort>, AppError> {
    favorite_service::list(&state.data_dir)
}

/// 新增收藏，同一端口不能重复。
#[tauri::command]
pub fn add_favorite(
    state: State<'_, AppState>,
    port: u16,
    protocol: String,
    note: String,
) -> Result<Vec<FavoritePort>, AppError> {
    favorite_service::add(&state.data_dir, port, protocol, note)
}

/// 更新收藏备注。
#[tauri::command]
pub fn update_favorite_note(
    state: State<'_, AppState>,
    port: u16,
    note: String,
) -> Result<Vec<FavoritePort>, AppError> {
    favorite_service::update_note(&state.data_dir, port, note)
}

/// 删除收藏。
#[tauri::command]
pub fn remove_favorite(state: State<'_, AppState>, port: u16) -> Result<Vec<FavoritePort>, AppError> {
    favorite_service::remove(&state.data_dir, port)
}
