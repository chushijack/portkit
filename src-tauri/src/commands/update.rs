//! 文件名称：update.rs
//!
//! 文件功能：
//! 应用更新相关 Tauri 命令。
//!
//! 主要职责：
//! - 接收前端调用
//! - 转发给更新服务
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-25

use tauri::AppHandle;

use crate::errors::AppError;
use crate::models::UpdateInfo;
use crate::services::update_service;

/// 返回当前应用版本号。
#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    update_service::current_version(&app)
}

/// 检查 GitHub Release 是否有新版本。
#[tauri::command]
pub async fn check_update(app: AppHandle, locale: String) -> Result<UpdateInfo, AppError> {
    update_service::check(&app, &locale).await
}

/// 下载并安装更新，完成后重启应用。
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), AppError> {
    update_service::download_and_install(&app).await
}

/// 重启应用。
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    update_service::restart(&app);
}
