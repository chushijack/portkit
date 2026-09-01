//! 文件名称：port.rs
//!
//! 文件功能：
//! 端口扫描相关 Tauri 命令。
//!
//! 主要职责：
//! - 接收扫描请求并返回端口列表
//! - 返回常用端口预设
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use tauri::State;

use crate::config::COMMON_PORT_ITEMS;
use crate::errors::AppError;
use crate::models::{CommonPort, PortInfo, ScanRequest};
use crate::services::{history_service, port_service};
use crate::state::AppState;

/// 扫描本机监听端口。
#[tauri::command]
pub async fn scan_ports(
    state: State<'_, AppState>,
    request: ScanRequest,
) -> Result<Vec<PortInfo>, AppError> {
    let data_dir = state.data_dir.clone();
    let record_history = request.record_history;
    let ports = tauri::async_runtime::spawn_blocking(move || port_service::scan_ports(&request))
        .await
        .map_err(|error| AppError::SystemError(error.to_string()))??;

    if record_history {
        history_service::record_scan(&data_dir, ports.len())?;
    }
    Ok(ports)
}

/// 返回常用开发端口及软件备注。
#[tauri::command]
pub fn get_common_ports() -> Vec<CommonPort> {
    COMMON_PORT_ITEMS
        .iter()
        .map(|item| CommonPort {
            port: item.port,
            label: item.label.to_string(),
        })
        .collect()
}
