//! 文件名称：settings.rs
//!
//! 文件功能：
//! 定义应用持久化设置。
//!
//! 主要职责：
//! - 覆盖扫描、系统、外观、语言与自动更新相关选项
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

use super::port::ScanMode;

/// 外观主题。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

/// 界面语言。`system` 表示跟随操作系统。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LanguageMode {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
}

impl Default for LanguageMode {
    fn default() -> Self {
        Self::System
    }
}

/// 本地持久化的应用设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scan_mode: ScanMode,
    pub custom_start: u16,
    pub custom_end: u16,
    pub refresh_interval: u32,
    pub auto_scan_on_startup: bool,
    pub autostart: bool,
    pub background_run: bool,
    pub notifications: bool,
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default)]
    pub locale: LanguageMode,
    /// 启动时是否自动检查 GitHub Release 更新。旧配置缺字段时默认开启。
    #[serde(default = "default_auto_check_update")]
    pub auto_check_update: bool,
}

fn default_auto_check_update() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_mode: ScanMode::All,
            custom_start: 3000,
            custom_end: 9000,
            refresh_interval: 3,
            auto_scan_on_startup: true,
            autostart: false,
            background_run: true,
            notifications: true,
            theme: ThemeMode::System,
            locale: LanguageMode::System,
            auto_check_update: true,
        }
    }
}
