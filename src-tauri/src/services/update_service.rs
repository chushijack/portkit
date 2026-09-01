//! 文件名称：update_service.rs
//!
//! 文件功能：
//! 应用自动更新的业务逻辑。
//!
//! 主要职责：
//! - 读取当前版本
//! - 向 GitHub Release 的 latest.json 检查更新
//! - 下载、安装更新并重启应用
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-25

use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

use crate::errors::AppError;
use crate::models::{UpdateInfo, UpdateProgress};
use crate::services::release_service;

/// 从 latest.json 的 notesLocalized 字段读取指定语言说明。
fn localized_notes_from_manifest(manifest: &serde_json::Value, locale: &str) -> Option<String> {
    let localized = manifest.get("notesLocalized")?;
    let key = match locale {
        "zh-CN" => "zh-CN",
        "ja" => "ja",
        _ => "en",
    };
    let text = localized.get(key)?.as_str()?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// 解析更新说明：内嵌 changelog 优先，其次 latest.json 多语言字段，避免展示 GitHub 折叠 HTML。
fn resolve_update_notes(
    version: &str,
    remote_body: Option<&str>,
    remote_manifest: Option<&serde_json::Value>,
    locale: &str,
) -> String {
    if let Some(release) = release_service::find_release_by_version(version) {
        return release.format_release_notes(locale);
    }

    if let Some(manifest) = remote_manifest {
        if let Some(notes) = localized_notes_from_manifest(manifest, locale) {
            return notes;
        }
    }

    if let Some(body) = remote_body {
        let trimmed = body.trim();
        // latest.json 旧版可能写入 GitHub Release 正文（含 <details>），桌面端不应原样展示。
        if !trimmed.is_empty() && !trimmed.contains("<details>") {
            return trimmed.to_string();
        }
    }

    String::new()
}

/// 返回当前运行中的应用版本。
pub fn current_version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 检查是否有可用更新。
///
/// 无新版本时 `available` 为 false，不视为错误。
pub async fn check(app: &AppHandle, locale: &str) -> Result<UpdateInfo, AppError> {
    let current = current_version(app);
    let update = app
        .updater()
        .map_err(|error| AppError::UpdateCheckFailed(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::UpdateCheckFailed(error.to_string()))?;

    match update {
        None => Ok(UpdateInfo {
            available: false,
            current_version: current.clone(),
            latest_version: current,
            notes: String::new(),
            date: None,
        }),
        Some(update) => Ok(UpdateInfo {
            available: true,
            current_version: update.current_version.clone(),
            latest_version: update.version.clone(),
            notes: resolve_update_notes(
                &update.version,
                update.body.as_deref(),
                Some(&update.raw_json),
                locale,
            ),
            date: update.date.map(|value| value.to_string()),
        }),
    }
}

/// 下载最新安装包、安装并重启。
///
/// Windows 安装器会自行退出并拉起新版本；便携版 zip 走自替换脚本；macOS 在安装完成后由本函数重启。
pub async fn download_and_install(app: &AppHandle) -> Result<(), AppError> {
    let update = app
        .updater()
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;

    let Some(update) = update else {
        return Err(AppError::UpdateNotAvailable);
    };

    #[cfg(windows)]
    if crate::system::is_portable_installation() {
        return crate::services::portable_update_service::download_and_install(app, &update.version)
            .await;
    }

    let handle = app.clone();
    let mut downloaded: u64 = 0;
    update
        .download_and_install(
            move |chunk_length, content_length| {
                downloaded += chunk_length as u64;
                let _ = handle.emit(
                    "update-download-progress",
                    UpdateProgress {
                        downloaded,
                        total: content_length,
                    },
                );
            },
            {
                let handle = app.clone();
                move || {
                    let _ = handle.emit("update-download-finished", ());
                }
            },
        )
        .await
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    app.restart();
}

/// 立即重启应用，用于安装完成后加载新版本。
pub fn restart(app: &AppHandle) {
    app.restart();
}
