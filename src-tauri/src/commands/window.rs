//! 文件名称：window.rs
//!
//! 文件功能：
//! 主窗口与快速搜索窗口的显示控制。
//!
//! 主要职责：
//! - 显示/聚焦主窗口
//! - 切换快速搜索窗口
//! - 打开设置页时通知前端
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::{AppHandle, Emitter, Manager};

use crate::errors::AppError;

/// 显示并聚焦主窗口。
pub fn show_main_window(app: &AppHandle) -> Result<(), AppError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::SystemError("主窗口不存在".to_string()))?;
    window
        .unminimize()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    window
        .show()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    Ok(())
}

/// 切换快速搜索窗口可见性。
pub fn toggle_quick_search_window(app: &AppHandle) -> Result<(), AppError> {
    let window = app
        .get_webview_window("quick-search")
        .ok_or_else(|| AppError::SystemError("快速搜索窗口不存在".to_string()))?;
    if window.is_visible().unwrap_or(false) {
        window
            .hide()
            .map_err(|error| AppError::SystemError(error.to_string()))?;
    } else {
        window
            .show()
            .map_err(|error| AppError::SystemError(error.to_string()))?;
        window
            .set_focus()
            .map_err(|error| AppError::SystemError(error.to_string()))?;
    }
    Ok(())
}

/// 隐藏快速搜索窗口。
#[tauri::command]
pub fn hide_quick_search(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("quick-search") {
        window
            .hide()
            .map_err(|error| AppError::SystemError(error.to_string()))?;
    }
    Ok(())
}

/// 打开主窗口并跳转到指定路由。
#[tauri::command]
pub fn open_main_route(app: AppHandle, route: String) -> Result<(), AppError> {
    show_main_window(&app)?;
    app.emit("navigate", route)
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    if let Some(window) = app.get_webview_window("quick-search") {
        let _ = window.hide();
    }
    Ok(())
}

/// 托盘「扫描端口」通知前端立即扫描。
pub fn emit_tray_scan(app: &AppHandle) -> Result<(), AppError> {
    show_main_window(app)?;
    app.emit("tray-action", "scan")
        .map_err(|error| AppError::SystemError(error.to_string()))
}
