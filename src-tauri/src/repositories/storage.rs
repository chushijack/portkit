//! 文件名称：storage.rs
//!
//! 文件功能：
//! 通用 JSON 文件读写，避免各仓储重复实现。
//!
//! 主要职责：
//! - 确保数据目录存在
//! - 读写 serde 可序列化结构
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::AppError;

/// 确保目录存在后返回 JSON 文件路径。
pub fn file_path(data_dir: &Path, file_name: &str) -> Result<PathBuf, AppError> {
    fs::create_dir_all(data_dir).map_err(|error| AppError::DataDir(error.to_string()))?;
    Ok(data_dir.join(file_name))
}

/// 读取 JSON 文件，文件不存在时返回默认值。
pub fn read_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T, AppError> {
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = fs::read_to_string(path).map_err(|error| AppError::Storage(error.to_string()))?;
    serde_json::from_str(&raw).map_err(|error| AppError::Storage(error.to_string()))
}

/// 将数据写入 JSON 文件。
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let raw =
        serde_json::to_string_pretty(value).map_err(|error| AppError::Storage(error.to_string()))?;
    fs::write(path, raw).map_err(|error| AppError::Storage(error.to_string()))
}
