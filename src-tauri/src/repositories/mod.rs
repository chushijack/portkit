//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出本地 JSON 仓储。
//!
//! 主要职责：
//! - 暴露设置、收藏、历史的持久化入口
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod favorite_repo;
pub mod history_repo;
pub mod settings_repo;
pub mod storage;

pub use favorite_repo::FavoriteRepository;
pub use history_repo::HistoryRepository;
pub use settings_repo::SettingsRepository;
