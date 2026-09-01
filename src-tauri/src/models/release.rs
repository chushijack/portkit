//! 文件名称：release.rs
//!
//! 文件功能：
//! 定义 GitHub Release 与更新日志数据结构。
//!
//! 主要职责：
//! - 约束更新日志页面展示数据
//! - 支持 zh-CN / en / ja 三语内容
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-26

use serde::{Deserialize, Serialize};

/// 更新日志支持的语言。
pub const CHANGELOG_LOCALES: [&str; 3] = ["zh-CN", "en", "ja"];

/// 更新日志分组类型，与官网 changelog 页面一致。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangelogSectionType {
    Added,
    Improved,
    Fixed,
    Other,
}

/// 三语文本，例如 release notes 摘要。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalizedNote {
    #[serde(rename = "zh-CN")]
    pub zh_cn: String,
    pub en: String,
    pub ja: String,
}

impl LocalizedNote {
    /// 按语言代码返回对应文本，未知语言回退到英文。
    pub fn resolve(&self, locale: &str) -> &str {
        match locale {
            "zh-CN" => &self.zh_cn,
            "ja" => &self.ja,
            _ => &self.en,
        }
    }
}

/// 三语条目列表。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalizedItems {
    #[serde(rename = "zh-CN")]
    pub zh_cn: Vec<String>,
    pub en: Vec<String>,
    pub ja: Vec<String>,
}

impl LocalizedItems {
    /// 按语言代码返回对应条目，未知语言回退到英文。
    pub fn resolve(&self, locale: &str) -> &[String] {
        match locale {
            "zh-CN" => &self.zh_cn,
            "ja" => &self.ja,
            _ => &self.en,
        }
    }
}

/// 单个版本的更新说明分组。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogSection {
    /// 分组类型：新增、优化、修复或其他。
    #[serde(rename = "type")]
    pub section_type: ChangelogSectionType,
    /// 各语言下的条目列表。
    pub items: LocalizedItems,
}

/// 一条 GitHub Release 记录，供更新日志页面展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRelease {
    /// Release 标签，例如 v0.9.12。
    pub tag: String,
    /// 去掉 v 前缀的版本号。
    pub version: String,
    /// Release 标题。
    pub name: String,
    /// ISO 8601 发布时间。
    pub published_at: String,
    /// GitHub Release 页面链接。
    pub html_url: String,
    /// 各语言 Release 说明摘要。
    pub notes: LocalizedNote,
    /// 解析后的分组说明。
    pub sections: Vec<ChangelogSection>,
}

/// 按语言解析后的更新日志，供前端直接展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAppRelease {
    pub tag: String,
    pub version: String,
    pub name: String,
    pub published_at: String,
    pub html_url: String,
    pub notes: String,
    pub sections: Vec<ResolvedChangelogSection>,
}

/// 按语言解析后的更新说明分组。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedChangelogSection {
    #[serde(rename = "type")]
    pub section_type: ChangelogSectionType,
    pub items: Vec<String>,
}

fn section_heading(locale: &str, section_type: &ChangelogSectionType) -> &'static str {
    match (locale, section_type) {
        ("zh-CN", ChangelogSectionType::Added) => "### 新增",
        ("zh-CN", ChangelogSectionType::Improved) => "### 优化",
        ("zh-CN", ChangelogSectionType::Fixed) => "### 修复",
        ("zh-CN", ChangelogSectionType::Other) => "### 说明",
        ("ja", ChangelogSectionType::Added) => "### 新機能",
        ("ja", ChangelogSectionType::Improved) => "### 改善",
        ("ja", ChangelogSectionType::Fixed) => "### 修正",
        ("ja", ChangelogSectionType::Other) => "### その他",
        (_, ChangelogSectionType::Added) => "### Added",
        (_, ChangelogSectionType::Improved) => "### Improved",
        (_, ChangelogSectionType::Fixed) => "### Fixed",
        (_, ChangelogSectionType::Other) => "### Other",
    }
}

impl AppRelease {
    /// 将多语言 changelog 渲染为纯文本更新说明，供更新弹窗展示。
    pub fn format_release_notes(&self, locale: &str) -> String {
        let resolved = self.resolve(locale);
        let mut parts: Vec<String> = Vec::new();

        if !resolved.notes.trim().is_empty() {
            parts.push(resolved.notes.trim().to_string());
        }

        for section in resolved.sections {
            if section.items.is_empty() {
                continue;
            }
            parts.push(section_heading(locale, &section.section_type).to_string());
            for item in section.items {
                parts.push(format!("- {item}"));
            }
        }

        parts.join("\n")
    }

    /// 将多语言 changelog 解析为指定语言的展示结构。
    pub fn resolve(&self, locale: &str) -> ResolvedAppRelease {
        ResolvedAppRelease {
            tag: self.tag.clone(),
            version: self.version.clone(),
            name: self.name.clone(),
            published_at: self.published_at.clone(),
            html_url: self.html_url.clone(),
            notes: self.notes.resolve(locale).to_string(),
            sections: self
                .sections
                .iter()
                .map(|section| ResolvedChangelogSection {
                    section_type: section.section_type.clone(),
                    items: section.items.resolve(locale).to_vec(),
                })
                .collect(),
        }
    }
}

/// 校验内嵌 changelog 是否包含完整三语结构。
pub fn validate_embedded_releases(releases: &[AppRelease]) -> Result<(), String> {
    for release in releases {
        for locale in CHANGELOG_LOCALES {
            if release.notes.resolve(locale).trim().is_empty() {
                return Err(format!(
                    "release {} has incomplete localized notes for {locale}",
                    release.tag
                ));
            }
        }

        if release.sections.is_empty() {
            return Err(format!("release {} has no changelog sections", release.tag));
        }

        for section in &release.sections {
            let counts: Vec<usize> = CHANGELOG_LOCALES
                .iter()
                .map(|locale| section.items.resolve(locale).len())
                .collect();
            if counts.iter().any(|count| *count == 0) {
                return Err(format!(
                    "release {} section {:?} has empty localized items",
                    release.tag, section.section_type
                ));
            }
            if counts.windows(2).any(|pair| pair[0] != pair[1]) {
                return Err(format!(
                    "release {} section {:?} item counts differ across locales",
                    release.tag, section.section_type
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_release() -> AppRelease {
        AppRelease {
            tag: "v0.9.17".to_string(),
            version: "0.9.17".to_string(),
            name: "PortKit v0.9.17".to_string(),
            published_at: "2026-08-29T12:00:00Z".to_string(),
            html_url: "https://github.com/chushijack/portkit/releases/tag/v0.9.17".to_string(),
            notes: LocalizedNote {
                zh_cn: "中文摘要".to_string(),
                en: "English summary".to_string(),
                ja: "日本語要約".to_string(),
            },
            sections: vec![ChangelogSection {
                section_type: ChangelogSectionType::Fixed,
                items: LocalizedItems {
                    zh_cn: vec!["修复问题".to_string()],
                    en: vec!["Fix issue".to_string()],
                    ja: vec!["問題を修正".to_string()],
                },
            }],
        }
    }

    #[test]
    fn resolve_release_by_locale() {
        let release = sample_release();
        let resolved = release.resolve("zh-CN");
        assert_eq!(resolved.notes, "中文摘要");
        assert_eq!(resolved.sections[0].items, vec!["修复问题".to_string()]);

        let fallback = release.resolve("fr");
        assert_eq!(fallback.notes, "English summary");
    }

    #[test]
    fn validate_embedded_release_structure() {
        validate_embedded_releases(&[sample_release()]).expect("sample release should be valid");
    }

    #[test]
    fn format_release_notes_by_locale() {
        let release = sample_release();
        let notes = release.format_release_notes("zh-CN");
        assert!(notes.contains("### 修复"));
        assert!(notes.contains("- 修复问题"));
    }
}
