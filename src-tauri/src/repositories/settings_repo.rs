//! 文件名称：settings_repo.rs
//!
//! 文件功能：
//! 读写应用设置 JSON。
//!
//! 主要职责：
//! - 加载设置，缺失时使用默认值
//! - 保存完整设置对象
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;

use crate::errors::AppError;
use crate::models::AppSettings;
use crate::repositories::storage;

const FILE_NAME: &str = "settings.json";

/// 设置仓储。
pub struct SettingsRepository;

impl SettingsRepository {
    /// 读取本地设置。
    pub fn load(data_dir: &Path) -> Result<AppSettings, AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        storage::read_json(&path)
    }

    /// 覆盖写入设置。
    pub fn save(data_dir: &Path, settings: &AppSettings) -> Result<(), AppError> {
        let path = storage::file_path(data_dir, FILE_NAME)?;
        storage::write_json(&path, settings)
    }
}
