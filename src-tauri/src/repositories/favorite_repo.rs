//! 文件名称：favorite_repo.rs
//!
//! 文件功能：
//! 读写收藏端口列表。
//!
//! 主要职责：
//! - 按端口号唯一保存，禁止重复
//! - 支持更新备注与删除
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::collections::HashSet;
use std::path::Path;

use crate::errors::AppError;
use crate::models::FavoritePort;
use crate::repositories::storage;

const FILE_NAME: &str = "favorites.json";

/// 收藏仓储。
pub struct FavoriteRepository;

impl FavoriteRepository {
    /// 读取全部收藏，同一端口只保留一条。
    pub fn list(data_dir: &Path) -> Result<Vec<FavoritePort>, AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        let items: Vec<FavoritePort> = storage::read_json(&path)?;
        Ok(dedupe_by_port(items))
    }

    /// 新增收藏；端口已存在时返回错误。
    pub fn insert(data_dir: &Path, favorite: FavoritePort) -> Result<Vec<FavoritePort>, AppError> {
        let mut items = Self::list(data_dir)?;
        if items.iter().any(|item| item.port == favorite.port) {
            return Err(AppError::FavoriteAlreadyExists);
        }
        items.insert(0, favorite);
        Self::persist(data_dir, &items)?;
        Ok(items)
    }

    /// 按端口更新备注。
    pub fn update_note(
        data_dir: &Path,
        port: u16,
        note: String,
    ) -> Result<Vec<FavoritePort>, AppError> {
        let mut items = Self::list(data_dir)?;
        let Some(existing) = items.iter_mut().find(|item| item.port == port) else {
            return Err(AppError::FavoriteNotFound);
        };
        existing.note = note;
        Self::persist(data_dir, &items)?;
        Ok(items)
    }

    /// 按端口删除收藏。
    pub fn remove(data_dir: &Path, port: u16) -> Result<Vec<FavoritePort>, AppError> {
        let mut items = Self::list(data_dir)?;
        items.retain(|item| item.port != port);
        Self::persist(data_dir, &items)?;
        Ok(items)
    }

    fn persist(data_dir: &Path, items: &[FavoritePort]) -> Result<(), AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        storage::write_json(&path, &items)
    }
}

fn dedupe_by_port(items: Vec<FavoritePort>) -> Vec<FavoritePort> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.port))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dedupe_by_port;
    use crate::models::FavoritePort;

    #[test]
    fn keeps_first_record_for_same_port() {
        let items = vec![
            FavoritePort {
                port: 5173,
                protocol: "TCP".to_string(),
                note: "first".to_string(),
                created_at: String::new(),
            },
            FavoritePort {
                port: 5173,
                protocol: "UDP".to_string(),
                note: "second".to_string(),
                created_at: String::new(),
            },
            FavoritePort {
                port: 8080,
                protocol: "TCP".to_string(),
                note: "api".to_string(),
                created_at: String::new(),
            },
        ];
        let unique = dedupe_by_port(items);
        assert_eq!(unique.len(), 2);
        assert_eq!(unique[0].note, "first");
        assert_eq!(unique[1].port, 8080);
    }
}
