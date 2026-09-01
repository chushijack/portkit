//! 文件名称：favorite_service.rs
//!
//! 文件功能：
//! 收藏端口的业务操作。
//!
//! 主要职责：
//! - 列出、新增、更新备注、删除收藏
//! - 同一端口只允许一条收藏
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;

use crate::errors::AppError;
use crate::models::FavoritePort;
use crate::repositories::FavoriteRepository;
use crate::utils::now_local_datetime;

/// 列出全部收藏。
pub fn list(data_dir: &Path) -> Result<Vec<FavoritePort>, AppError> {
    FavoriteRepository::list(data_dir)
}

/// 新增收藏，端口已存在时拒绝。
pub fn add(
    data_dir: &Path,
    port: u16,
    protocol: String,
    note: String,
) -> Result<Vec<FavoritePort>, AppError> {
    let favorite = build_favorite(port, protocol, note)?;
    FavoriteRepository::insert(data_dir, favorite)
}

/// 更新已有收藏的备注。
pub fn update_note(data_dir: &Path, port: u16, note: String) -> Result<Vec<FavoritePort>, AppError> {
    if port == 0 {
        return Err(AppError::InvalidPort);
    }
    FavoriteRepository::update_note(data_dir, port, note.trim().to_string())
}

/// 取消收藏。
pub fn remove(data_dir: &Path, port: u16) -> Result<Vec<FavoritePort>, AppError> {
    FavoriteRepository::remove(data_dir, port)
}

fn build_favorite(port: u16, protocol: String, note: String) -> Result<FavoritePort, AppError> {
    if port == 0 {
        return Err(AppError::InvalidPort);
    }
    Ok(FavoritePort {
        port,
        protocol: normalize_protocol(&protocol)?,
        note: note.trim().to_string(),
        created_at: now_local_datetime(),
    })
}

fn normalize_protocol(protocol: &str) -> Result<String, AppError> {
    match protocol.trim().to_ascii_uppercase().as_str() {
        "TCP" => Ok("TCP".to_string()),
        "UDP" => Ok("UDP".to_string()),
        _ => Err(AppError::InvalidProtocol),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_protocol;

    #[test]
    fn accepts_tcp_and_udp_case_insensitive() {
        assert_eq!(normalize_protocol("tcp").ok().as_deref(), Some("TCP"));
        assert_eq!(normalize_protocol("UDP").ok().as_deref(), Some("UDP"));
    }

    #[test]
    fn rejects_unknown_protocol() {
        assert!(normalize_protocol("HTTP").is_err());
    }
}
