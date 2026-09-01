//! 文件名称：release_service.rs
//!
//! 文件功能：
//! 提供更新日志列表，数据来自本地 docs/changelog。
//!
//! 主要职责：
//! - 读取编译时内嵌的各版本 release.json
//! - 校验三语结构并按界面语言解析
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-26

use crate::models::release::{validate_embedded_releases, AppRelease, ResolvedAppRelease};

const EMBEDDED_CHANGELOG: &str = include_str!(concat!(env!("OUT_DIR"), "/embedded_changelog.json"));

fn load_releases() -> Vec<AppRelease> {
    let releases: Vec<AppRelease> =
        serde_json::from_str(EMBEDDED_CHANGELOG).unwrap_or_default();
    if let Err(error) = validate_embedded_releases(&releases) {
        eprintln!("embedded changelog validation failed: {error}");
    }
    releases
}

/// 返回内嵌的本地更新日志列表。
pub async fn list_releases() -> Vec<AppRelease> {
    load_releases()
}

/// 按界面语言返回解析后的更新日志列表。
pub async fn list_resolved_releases(locale: &str) -> Vec<ResolvedAppRelease> {
    load_releases()
        .into_iter()
        .map(|release| release.resolve(locale))
        .collect()
}

/// 按 semver 版本号查找内嵌 changelog；用于 updater 远端 notes 缺失时的回退。
pub fn find_release_by_version(version: &str) -> Option<AppRelease> {
    load_releases()
        .into_iter()
        .find(|release| release.version == version)
}
