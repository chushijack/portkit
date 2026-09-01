//! 文件名称：mod.rs
//!
//! 文件功能：
//! 导出应用错误类型。
//!
//! 主要职责：
//! - 暴露 AppError
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

pub mod app_error;

pub use app_error::AppError;
