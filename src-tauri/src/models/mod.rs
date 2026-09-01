//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出前后端共享的数据结构。
//!
//! 主要职责：
//! - 暴露端口、进程、收藏、历史、设置、更新模型
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod favorite;
pub mod history;
pub mod kill;
pub mod port;
pub mod process;
pub mod release;
pub mod settings;
pub mod update;

pub use favorite::FavoritePort;
pub use history::HistoryRecord;
pub use kill::{is_docker_process, KillMode, KillOutcome, ProcessController, ProcessIdentity};
pub use port::{CommonPort, PortInfo, ScanMode, ScanRequest};
pub use process::ProcessDetail;
pub use settings::{AppSettings, LanguageMode};
pub use update::{UpdateInfo, UpdateProgress};
