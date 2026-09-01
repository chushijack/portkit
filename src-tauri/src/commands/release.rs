//! 文件名称：release.rs
//!
//! 文件功能：
//! 更新日志相关 Tauri 命令。
//!
//! 主要职责：
//! - 向前端提供 GitHub Release 列表
//! - 按界面语言返回解析后的更新日志
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-26

use crate::models::release::{AppRelease, ResolvedAppRelease};
use crate::services::release_service;

/// 返回完整三语更新日志列表。
#[tauri::command]
pub async fn get_releases() -> Vec<AppRelease> {
    release_service::list_releases().await
}

/// 按界面语言返回解析后的更新日志列表。
#[tauri::command]
pub async fn get_resolved_releases(locale: String) -> Vec<ResolvedAppRelease> {
    release_service::list_resolved_releases(&locale).await
}
