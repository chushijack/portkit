/**
 * 文件名称：changelog-locales.mjs
 *
 * 文件功能：
 * 更新日志多语言常量与校验工具。
 *
 * 主要职责：
 * - 定义 changelog 支持的语言列表
 * - 校验 release.json 多语言结构完整性
 * - 按语言解析 sections 条目
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-29
 */

/** @typedef {"zh-CN" | "en" | "ja"} ChangelogLocale */

/** @type {readonly ChangelogLocale[]} */
export const CHANGELOG_LOCALES = ["zh-CN", "en", "ja"];

/** GitHub Release 默认展示语言。 */
export const DEFAULT_RELEASE_LOCALE = "en";

/** @type {Record<ChangelogLocale, string>} */
export const LOCALE_LABELS = {
  "zh-CN": "## 中文",
  en: "## English",
  ja: "## 日本語",
};

/** GitHub Release 折叠区块标题。 */
export const LOCALE_SUMMARY_LABELS = {
  "zh-CN": "中文",
  en: "English",
  ja: "日本語",
};

/** @type {Record<ChangelogLocale, Record<string, string>>} */
export const SECTION_HEADINGS = {
  "zh-CN": {
    added: "### 新增",
    improved: "### 优化",
    fixed: "### 修复",
    other: "### 说明",
  },
  en: {
    added: "### Added",
    improved: "### Improved",
    fixed: "### Fixed",
    other: "### Other",
  },
  ja: {
    added: "### 新機能",
    improved: "### 改善",
    fixed: "### 修正",
    other: "### その他",
  },
};

/**
 * @param {unknown} value
 * @returns {value is Record<ChangelogLocale, string>}
 */
export function isLocalizedNote(value) {
  if (!value || typeof value !== "object") {
    return false;
  }
  return CHANGELOG_LOCALES.every((locale) => {
    const text = /** @type {Record<string, unknown>} */ (value)[locale];
    return typeof text === "string" && text.trim().length > 0;
  });
}

/**
 * @param {unknown} value
 * @returns {value is Record<ChangelogLocale, string[]>}
 */
export function isLocalizedItems(value) {
  if (!value || typeof value !== "object") {
    return false;
  }
  return CHANGELOG_LOCALES.every((locale) => {
    const items = /** @type {Record<string, unknown>} */ (value)[locale];
    return (
      Array.isArray(items) &&
      items.length > 0 &&
      items.every((item) => typeof item === "string" && item.trim().length > 0)
    );
  });
}

/**
 * @param {{ tag?: string, notes?: unknown, sections?: Array<{ type?: string, items?: unknown }> }} release
 * @param {string} [fileLabel]
 */
export function validateRelease(release, fileLabel = "release.json") {
  if (!release.tag?.trim()) {
    throw new Error(`Invalid ${fileLabel}: missing tag`);
  }
  if (!isLocalizedNote(release.notes)) {
    throw new Error(`Invalid ${fileLabel}: notes must include zh-CN, en, and ja`);
  }
  if (!release.sections?.length) {
    throw new Error(`Invalid ${fileLabel}: sections must not be empty`);
  }

  for (const [index, section] of release.sections.entries()) {
    if (!section.type?.trim()) {
      throw new Error(`Invalid ${fileLabel}: section[${index}] missing type`);
    }
    if (!isLocalizedItems(section.items)) {
      throw new Error(
        `Invalid ${fileLabel}: section[${index}] items must include non-empty zh-CN, en, and ja arrays`,
      );
    }

    const counts = CHANGELOG_LOCALES.map((locale) => section.items[locale].length);
    if (new Set(counts).size !== 1) {
      throw new Error(
        `Invalid ${fileLabel}: section[${index}] item counts differ across locales (${counts.join(", ")})`,
      );
    }
  }
}

/**
 * @param {Record<ChangelogLocale, string[]>} items
 * @param {ChangelogLocale} locale
 */
export function resolveItems(items, locale) {
  return items[locale];
}
