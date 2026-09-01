//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出通用工具函数。
//!
//! 主要职责：
//! - 暴露时长、时间格式化与界面语言解析
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod locale;
pub mod time;

pub use locale::{resolve_ui_locale, UiLocale};
pub use time::{format_local_datetime, format_uptime, now_local_datetime};
