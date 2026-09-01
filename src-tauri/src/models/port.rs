//! 文件名称：port.rs
//!
//! 文件功能：
//! 定义端口扫描请求与列表展示模型。
//!
//! 主要职责：
//! - 描述扫描范围
//! - 描述前端表格所需的端口字段
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

/// 端口扫描范围模式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScanMode {
    All,
    Common,
    Custom,
}

impl Default for ScanMode {
    fn default() -> Self {
        Self::All
    }
}

/// 一次扫描请求。自定义模式需要起止端口。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub mode: ScanMode,
    pub start_port: Option<u16>,
    pub end_port: Option<u16>,
    /// 手动扫描才写入历史，自动刷新不记，避免刷屏。
    #[serde(default)]
    pub record_history: bool,
}

/// 端口列表行数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub status: String,
    pub listen_address: String,
    pub process_name: String,
    pub pid: u32,
    pub occupied_time: String,
    pub occupied_seconds: u64,
    /// Unix 秒；`0` 表示没读到启动时间，列表排序时放到最后。
    pub start_time: u64,
    pub service_label: Option<String>,
    pub command: Option<String>,
}

/// 常用端口预设，供底栏菜单展示端口与软件备注。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonPort {
    pub port: u16,
    pub label: String,
}
