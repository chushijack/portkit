//! 文件名称：locale.rs
//!
//! 文件功能：
//! 根据设置与操作系统解析界面语言。
//!
//! 主要职责：
//! - 跟随系统时读取 OS 语言
//! - 映射为中文、英文或日文
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use crate::models::LanguageMode;

/// 托盘和系统文案实际使用的语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLocale {
    ZhCn,
    En,
    Ja,
}

/// 设置项为跟随系统时读取 OS 语言，否则使用指定语言。
pub fn resolve_ui_locale(mode: &LanguageMode) -> UiLocale {
    match mode {
        LanguageMode::ZhCn => UiLocale::ZhCn,
        LanguageMode::En => UiLocale::En,
        LanguageMode::Ja => UiLocale::Ja,
        LanguageMode::System => detect_system_locale(),
    }
}

fn detect_system_locale() -> UiLocale {
    let tag = os_locale_tag().to_lowercase().replace('_', "-");
    if tag.starts_with("zh") {
        UiLocale::ZhCn
    } else if tag.starts_with("ja") {
        UiLocale::Ja
    } else {
        UiLocale::En
    }
}

#[cfg(windows)]
fn os_locale_tag() -> String {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;

    let mut buffer = [0u16; 85];
    let len = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if len > 1 {
        String::from_utf16_lossy(&buffer[..(len as usize - 1)])
    } else {
        "en".to_string()
    }
}

#[cfg(not(windows))]
fn os_locale_tag() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "en".to_string())
}
