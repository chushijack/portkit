//! 文件名称：favorite.rs
//!
//! 文件功能：
//! 定义收藏端口记录。
//!
//! 主要职责：
//! - 保存端口、协议与开发者备注
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

/// 用户收藏的常用端口。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritePort {
    pub port: u16,
    pub protocol: String,
    pub note: String,
    pub created_at: String,
}
