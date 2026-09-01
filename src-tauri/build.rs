//! 文件名称：build.rs
//!
//! 文件功能：
//! Tauri 构建脚本，生成上下文与内嵌更新日志。
//!
//! 主要职责：
//! - 在编译前调用 tauri_build
//! - 将 docs/changelog/<tag>/release.json 合并为内嵌 JSON
//! - 校验 changelog 三语结构完整
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::env;
use std::fs;
use std::path::Path;

const CHANGELOG_LOCALES: [&str; 3] = ["zh-CN", "en", "ja"];

fn main() {
    embed_changelog();
    tauri_build::build();
}

/// 校验 release.json 是否包含完整三语 notes 与 sections。
fn validate_release_json(value: &serde_json::Value, context: &str) {
    validate_localized_note(
        value.get("notes").unwrap_or(&serde_json::Value::Null),
        &format!("{context}: notes"),
    );

    let empty_sections: Vec<serde_json::Value> = Vec::new();
    let sections = value
        .get("sections")
        .and_then(|value| value.as_array())
        .unwrap_or(&empty_sections);
    assert!(!sections.is_empty(), "{context}: sections must not be empty");

    for (index, section) in sections.iter().enumerate() {
        let section_type = section
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        assert!(
            !section_type.trim().is_empty(),
            "{context}: sections[{index}] missing type"
        );

        let items = section.get("items").unwrap_or(&serde_json::Value::Null);
        let counts = validate_localized_items(items, &format!("{context}: sections[{index}].items"));
        assert_eq!(
            counts[0], counts[1],
            "{context}: sections[{index}] zh-CN/en item counts differ"
        );
        assert_eq!(
            counts[1], counts[2],
            "{context}: sections[{index}] en/ja item counts differ"
        );
    }
}

fn validate_localized_note(value: &serde_json::Value, context: &str) {
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"));
    for locale in CHANGELOG_LOCALES {
        let text = object
            .get(locale)
            .and_then(|value| value.as_str())
            .unwrap_or("");
        assert!(
            !text.trim().is_empty(),
            "{context}.{locale} must be a non-empty string"
        );
    }
}

fn validate_localized_items(value: &serde_json::Value, context: &str) -> [usize; 3] {
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"));
    let mut counts = [0usize; 3];

    for (index, locale) in CHANGELOG_LOCALES.iter().enumerate() {
        let items = object
            .get(*locale)
            .and_then(|value| value.as_array())
            .unwrap_or_else(|| panic!("{context}.{locale} must be a non-empty array"));
        assert!(!items.is_empty(), "{context}.{locale} must not be empty");
        for item in items {
            let text = item.as_str().unwrap_or("");
            assert!(
                !text.trim().is_empty(),
                "{context}.{locale} items must be non-empty strings"
            );
        }
        counts[index] = items.len();
    }

    counts
}

/// 扫描版本目录，生成供 release_service 内嵌的 changelog 数组。
fn embed_changelog() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let changelog_root = Path::new(&manifest_dir).join("../docs/changelog");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let output_path = Path::new(&out_dir).join("embedded_changelog.json");

    println!("cargo:rerun-if-changed={}", changelog_root.display());

    let mut releases: Vec<serde_json::Value> = Vec::new();

    if changelog_root.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(&changelog_root)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .collect();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let release_path = entry.path().join("release.json");
            println!("cargo:rerun-if-changed={}", release_path.display());
            if !release_path.is_file() {
                continue;
            }
            let raw = match fs::read_to_string(&release_path) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let value = serde_json::from_str::<serde_json::Value>(&raw)
                .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", release_path.display()));
            validate_release_json(&value, &release_path.display().to_string());
            releases.push(value);
        }
    }

    releases.sort_by(|left, right| {
        let left_date = left
            .get("publishedAt")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let right_date = right
            .get("publishedAt")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        right_date.cmp(left_date)
    });

    let encoded = serde_json::to_string(&releases).unwrap_or_else(|_| "[]".to_string());
    fs::write(output_path, encoded).expect("write embedded changelog");
}
