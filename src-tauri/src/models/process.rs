//! 文件名称：process.rs
//!
//! 文件功能：
//! 定义进程详情页所需的数据结构。
//!
//! 主要职责：
//! - 汇总进程元数据、资源占用和关联端口
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

/// 进程详情，对应详情页卡片与性能条。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessDetail {
    pub pid: u32,
    pub name: String,
    pub display_name: String,
    /// 按占用端口识别的开发服务名，例如 Vite Dev Server。
    pub service_labels: Vec<String>,
    pub status: String,
    pub user: String,
    pub start_time: String,
    pub duration: String,
    pub priority: String,
    pub path: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
}
