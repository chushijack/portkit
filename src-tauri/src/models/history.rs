//! 文件名称：history.rs
//!
//! 文件功能：
//! 定义端口操作历史记录。
//!
//! 主要职责：
//! - 保存扫描与关闭端口等操作痕迹
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

/// 一条操作历史。扫描记录的端口字段可为空。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: String,
    pub time: String,
    pub action: String,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub process: Option<String>,
    pub pid: Option<u32>,
}
