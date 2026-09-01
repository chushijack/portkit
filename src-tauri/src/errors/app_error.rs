//! 文件名称：app_error.rs
//!
//! 文件功能：
//! 定义 PortKit 统一错误类型，供 Command 返回给前端。
//!
//! 主要职责：
//! - 覆盖权限、进程保护、系统调用失败等场景
//! - 序列化为稳定错误码，由前端按语言翻译
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::Serialize;

/// 应用层错误。序列化给前端的是错误码，文案由 i18n 负责。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未找到对应进程")]
    ProcessNotFound,
    #[error("权限不足，无法结束该进程。请尝试以管理员身份运行 PortKit")]
    #[allow(dead_code)]
    PermissionDenied,
    #[error("未能结束该进程，进程仍在运行")]
    ProcessStillRunning,
    #[error("该进程受系统保护，禁止结束，以免影响系统稳定")]
    ProtectedProcess,
    #[error("不能结束 PortKit 自身进程")]
    SelfProcess,
    #[error("扫描范围无效，请确认起始端口不大于结束端口")]
    InvalidScanRange,
    #[error("端口号无效，请输入 1-65535")]
    InvalidPort,
    #[error("协议仅支持 TCP 或 UDP")]
    InvalidProtocol,
    #[error("该端口已收藏，不能重复添加")]
    FavoriteAlreadyExists,
    #[error("未找到该收藏")]
    FavoriteNotFound,
    #[error("当前平台暂不支持该系统能力")]
    #[allow(dead_code)]
    UnsupportedPlatform,
    #[error("数据目录不可用：{0}")]
    DataDir(String),
    #[error("读写本地数据失败：{0}")]
    Storage(String),
    #[error("系统调用失败：{0}")]
    SystemError(String),
    #[error("检查更新失败：{0}")]
    UpdateCheckFailed(String),
    #[error("下载更新失败：{0}")]
    UpdateDownloadFailed(String),
    #[error("安装更新失败：{0}")]
    UpdateInstallFailed(String),
    #[error("当前已是最新版本")]
    UpdateNotAvailable,
}

impl AppError {
    fn payload(&self) -> String {
        match self {
            AppError::ProcessNotFound => "PROCESS_NOT_FOUND".to_string(),
            AppError::PermissionDenied => "PERMISSION_DENIED".to_string(),
            AppError::ProcessStillRunning => "PROCESS_STILL_RUNNING".to_string(),
            AppError::ProtectedProcess => "PROTECTED_PROCESS".to_string(),
            AppError::SelfProcess => "SELF_PROCESS".to_string(),
            AppError::InvalidScanRange => "INVALID_SCAN_RANGE".to_string(),
            AppError::InvalidPort => "INVALID_PORT".to_string(),
            AppError::InvalidProtocol => "INVALID_PROTOCOL".to_string(),
            AppError::FavoriteAlreadyExists => "FAVORITE_ALREADY_EXISTS".to_string(),
            AppError::FavoriteNotFound => "FAVORITE_NOT_FOUND".to_string(),
            AppError::UnsupportedPlatform => "UNSUPPORTED_PLATFORM".to_string(),
            AppError::DataDir(detail) => format!("DATA_DIR|{detail}"),
            AppError::Storage(detail) => format!("STORAGE|{detail}"),
            AppError::SystemError(detail) => format!("SYSTEM_ERROR|{detail}"),
            AppError::UpdateCheckFailed(detail) => format!("UPDATE_CHECK_FAILED|{detail}"),
            AppError::UpdateDownloadFailed(detail) => format!("UPDATE_DOWNLOAD_FAILED|{detail}"),
            AppError::UpdateInstallFailed(detail) => format!("UPDATE_INSTALL_FAILED|{detail}"),
            AppError::UpdateNotAvailable => "UPDATE_NOT_AVAILABLE".to_string(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.payload())
    }
}
