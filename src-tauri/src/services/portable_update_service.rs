//! 文件名称：portable_update_service.rs
//!
//! 文件功能：
//! Windows 便携版自更新：下载 release zip 并替换当前 exe。
//!
//! 主要职责：
//! - 从 GitHub Release 下载 PortKit_<version>_windows.zip
//! - 解压新版 exe，退出后由脚本覆盖并重启
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-29

use std::path::{Path, PathBuf};
use std::process::Command;

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::errors::AppError;
use crate::models::UpdateProgress;

const GITHUB_REPO: &str = "chushijack/portkit";

/// 下载便携包 zip、替换当前 exe 并退出；重启由临时脚本完成。
pub async fn download_and_install(app: &AppHandle, version: &str) -> Result<(), AppError> {
    let current_exe = std::env::current_exe()
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    let temp_dir = std::env::temp_dir().join(format!("portkit-update-{version}"));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;
    }
    std::fs::create_dir_all(&temp_dir)
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    let zip_path = temp_dir.join("update.zip");
    let download_url = portable_zip_url(version);
    download_with_progress(app, &download_url, &zip_path).await?;

    let new_exe = extract_portable_exe(&zip_path, &temp_dir)?;
    schedule_exe_replacement(&current_exe, &new_exe)?;

    app.exit(0);
    Ok(())
}

fn portable_zip_url(version: &str) -> String {
    format!(
        "https://github.com/{GITHUB_REPO}/releases/download/v{version}/PortKit_{version}_windows.zip"
    )
}

async fn download_with_progress(
    app: &AppHandle,
    url: &str,
    destination: &Path,
) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .user_agent("PortKit-Updater")
        .build()
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::UpdateDownloadFailed(format!(
            "HTTP {} for {url}",
            response.status()
        )));
    }

    let total = response.content_length();
    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(destination)
        .await
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;
        file.write_all(&chunk)
            .await
            .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;
        downloaded += chunk.len() as u64;
        let _ = app.emit(
            "update-download-progress",
            UpdateProgress {
                downloaded,
                total,
            },
        );
    }

    file.flush()
        .await
        .map_err(|error| AppError::UpdateDownloadFailed(error.to_string()))?;
    let _ = app.emit("update-download-finished", ());
    Ok(())
}

fn extract_portable_exe(zip_path: &Path, destination_dir: &Path) -> Result<PathBuf, AppError> {
    let archive_file = std::fs::File::open(zip_path)
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;
    let mut archive = zip::ZipArchive::new(archive_file)
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;
        let Some(file_name) = Path::new(entry.name()).file_name() else {
            continue;
        };
        if !is_portable_executable_name(file_name) {
            continue;
        }

        let output_path = destination_dir.join(file_name);
        let mut output_file = std::fs::File::create(&output_path)
            .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;
        std::io::copy(&mut entry, &mut output_file)
            .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;
        return Ok(output_path);
    }

    Err(AppError::UpdateInstallFailed(
        "portable zip does not contain PortKit.exe".to_string(),
    ))
}

fn is_portable_executable_name(file_name: &std::ffi::OsStr) -> bool {
    file_name
        .to_string_lossy()
        .eq_ignore_ascii_case("PortKit.exe")
}

fn schedule_exe_replacement(current_exe: &Path, new_exe: &Path) -> Result<(), AppError> {
    let pid = std::process::id();
    let script_path = std::env::temp_dir().join(format!("portkit-update-{pid}.ps1"));
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
Wait-Process -Id {pid} -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Copy-Item -LiteralPath '{new_exe}' -Destination '{current_exe}' -Force
Start-Process -FilePath '{current_exe}'
Remove-Item -LiteralPath '{script_path}' -Force
"#,
        new_exe = escape_powershell_literal(new_exe),
        current_exe = escape_powershell_literal(current_exe),
        script_path = escape_powershell_literal(&script_path),
    );

    std::fs::write(&script_path, script)
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
            script_path
                .to_str()
                .ok_or_else(|| AppError::UpdateInstallFailed("invalid script path".to_string()))?,
        ])
        .spawn()
        .map_err(|error| AppError::UpdateInstallFailed(error.to_string()))?;

    Ok(())
}

fn escape_powershell_literal(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}
