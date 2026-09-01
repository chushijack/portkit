//! 文件名称：history_repo.rs
//!
//! 文件功能：
//! 读写端口操作历史。
//!
//! 主要职责：
//! - 新记录插入到最前
//! - 超出上限时丢弃最旧记录
//! - 支持清空
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;

use crate::errors::AppError;
use crate::models::HistoryRecord;
use crate::repositories::storage;

const FILE_NAME: &str = "history.json";
const MAX_RECORDS: usize = 500;

/// 历史仓储。
pub struct HistoryRepository;

impl HistoryRepository {
    /// 读取全部历史，最新在前。
    pub fn list(data_dir: &Path) -> Result<Vec<HistoryRecord>, AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        storage::read_json(&path)
    }

    /// 追加一条历史。
    pub fn append(data_dir: &Path, record: HistoryRecord) -> Result<Vec<HistoryRecord>, AppError> {
        let mut items = Self::list(data_dir)?;
        items.insert(0, record);
        items.truncate(MAX_RECORDS);
        Self::persist(data_dir, &items)?;
        Ok(items)
    }

    /// 清空历史。
    pub fn clear(data_dir: &Path) -> Result<(), AppError> {
        Self::persist(data_dir, &[])
    }

    fn persist(data_dir: &Path, items: &[HistoryRecord]) -> Result<(), AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        storage::write_json(&path, &items)
    }
}
