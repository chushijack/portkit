//! 文件名称：history_service.rs
//!
//! 文件功能：
//! 端口操作历史的业务操作。
//!
//! 主要职责：
//! - 记录扫描和关闭端口
//! - 查询与清空历史
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;

use crate::errors::AppError;
use crate::models::HistoryRecord;
use crate::repositories::HistoryRepository;
use crate::utils::now_local_datetime;

/// 列出历史记录。
pub fn list(data_dir: &Path) -> Result<Vec<HistoryRecord>, AppError> {
    HistoryRepository::list(data_dir)
}

/// 记录一次扫描操作。只存端口数量，文案由前端按当前语言拼接。
pub fn record_scan(data_dir: &Path, port_count: usize) -> Result<(), AppError> {
    let record = HistoryRecord {
        id: unique_id("scan"),
        time: now_local_datetime(),
        action: "scan".to_string(),
        port: None,
        protocol: None,
        process: Some(port_count.to_string()),
        pid: None,
    };
    HistoryRepository::append(data_dir, record)?;
    Ok(())
}

/// 记录一次关闭端口操作。
pub fn record_kill(
    data_dir: &Path,
    port: Option<u16>,
    protocol: Option<String>,
    process: Option<String>,
    pid: u32,
) -> Result<(), AppError> {
    let record = HistoryRecord {
        id: unique_id("kill"),
        time: now_local_datetime(),
        action: "kill".to_string(),
        port,
        protocol,
        process,
        pid: Some(pid),
    };
    HistoryRepository::append(data_dir, record)?;
    Ok(())
}

/// 清空历史。
pub fn clear(data_dir: &Path) -> Result<(), AppError> {
    HistoryRepository::clear(data_dir)
}

fn unique_id(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        chrono::Local::now().timestamp_millis()
    )
}
