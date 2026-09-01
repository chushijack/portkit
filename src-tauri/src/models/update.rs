//! 文件名称：update.rs
//!
//! 文件功能：
//! 定义应用内自动更新的数据结构。
//!
//! 主要职责：
//! - 描述当前版本与远端最新版本
//! - 描述更新流程状态与下载进度
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-25

use serde::{Deserialize, Serialize};

/// 一次更新检查的结果，供前端展示版本号与更新日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// 是否存在高于当前版本的可用更新。
    pub available: bool,
    /// 当前安装的应用版本。
    pub current_version: String,
    /// 远端 latest.json 中的最新版本；无更新时与当前版本相同。
    pub latest_version: String,
    /// Release 更新说明，可能为空。
    pub notes: String,
    /// 发布时间；缺失时前端不展示。
    pub date: Option<String>,
}

/// 更新流程所处阶段，前后端共用同一组状态名。
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available,
    Downloading,
    Installing,
    UpToDate,
    Error,
}

/// 下载过程中的字节进度，通过事件推给前端。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    /// 已下载字节数。
    pub downloaded: u64,
    /// 资源总大小；服务端未提供 Content-Length 时为 None。
    pub total: Option<u64>,
}
