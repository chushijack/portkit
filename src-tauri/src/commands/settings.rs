//! 文件名称：settings.rs
//!
//! 文件功能：
//! 应用设置相关 Tauri 命令。
//!
//! 主要职责：
//! - 读取设置
//! - 保存设置、同步后台运行开关与托盘语言
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::{AppHandle, State};

use crate::errors::AppError;
use crate::models::AppSettings;
use crate::rebuild_tray;
use crate::services::settings_service;
use crate::state::AppState;
use crate::utils::resolve_ui_locale;

/// 读取应用设置。
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    settings_service::load(&state.data_dir)
}

/// 保存应用设置。
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    let saved = settings_service::save(&state.data_dir, settings)?;
    if let Ok(mut flag) = state.background_run.lock() {
        *flag = saved.background_run;
    }
    let _ = rebuild_tray(&app, resolve_ui_locale(&saved.locale));
    Ok(saved)
}
