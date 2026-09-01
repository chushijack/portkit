//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出业务服务。
//!
//! 主要职责：
//! - 暴露端口、进程、收藏、历史、设置、更新服务
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod favorite_service;
pub mod history_service;
pub mod port_service;
pub mod process_service;
pub mod release_service;
pub mod service_detector;
pub mod settings_service;
#[cfg(windows)]
pub mod portable_update_service;
pub mod update_service;
