//! 文件名称：state.rs
//!
//! 文件功能：
//! 应用运行时共享状态。
//!
//! 主要职责：
//! - 保存数据目录
//! - 保存关闭到托盘开关
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::PathBuf;
use std::sync::Mutex;

/// Tauri 全局状态。
pub struct AppState {
    /// 设置、收藏、历史 JSON 所在目录。
    pub data_dir: PathBuf,
    /// 关闭主窗口时是否隐藏到托盘。
    pub background_run: Mutex<bool>,
}
