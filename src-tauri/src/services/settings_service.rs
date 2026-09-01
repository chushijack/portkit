//! 文件名称：settings_service.rs
//!
//! 文件功能：
//! 设置读写与校验。
//!
//! 主要职责：
//! - 加载/保存设置
//! - 校正刷新间隔和自定义扫描范围
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;

use crate::errors::AppError;
use crate::models::AppSettings;
use crate::repositories::SettingsRepository;

const ALLOWED_INTERVALS: [u32; 4] = [1, 3, 5, 10];

/// 读取设置。
pub fn load(data_dir: &Path) -> Result<AppSettings, AppError> {
    SettingsRepository::load(data_dir)
}

/// 保存设置。非法刷新间隔会回退到 3 秒。
pub fn save(data_dir: &Path, mut settings: AppSettings) -> Result<AppSettings, AppError> {
    if !ALLOWED_INTERVALS.contains(&settings.refresh_interval) {
        settings.refresh_interval = 3;
    }
    if settings.custom_start == 0
        || settings.custom_end == 0
        || settings.custom_start > settings.custom_end
    {
        return Err(AppError::InvalidScanRange);
    }
    SettingsRepository::save(data_dir, &settings)?;
    Ok(settings)
}
