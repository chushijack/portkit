/**
 * 文件名称：render-release-notes.mjs
 *
 * 文件功能：
 * 将 docs/changelog/<tag>/release.json 渲染为 GitHub Release 正文。
 *
 * 主要职责：
 * - 供 CI 在发布前生成 release notes
 * - 校验 tag 与 release.json 一致
 * - 默认展示英文，其他语言放入可折叠区块（仅用于 GitHub Release 正文）
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-26
 */

import { readFileSync } from "node:fs";
import {
  CHANGELOG_LOCALES,
  DEFAULT_RELEASE_LOCALE,
  LOCALE_SUMMARY_LABELS,
  SECTION_HEADINGS,
  validateRelease,
} from "./changelog-locales.mjs";

/**
 * @param {{ notes: Record<string, string>, sections: Array<{ type: string, items: Record<string, string[]> }> }} release
 * @param {import("./changelog-locales.mjs").ChangelogLocale} locale
 */
function renderLocaleNotes(release, locale) {
  const parts = [];
  const headings = SECTION_HEADINGS[locale];

  if (release.notes[locale]?.trim()) {
    parts.push(release.notes[locale].trim(), "");
  }

  for (const section of release.sections ?? []) {
    const heading = headings[section.type] ?? headings.other;
    const items = section.items?.[locale] ?? [];
    if (!items.length) {
      continue;
    }
    parts.push(heading);
    for (const item of items) {
      parts.push(`- ${item}`);
    }
    parts.push("");
  }

  return parts.join("\n").trim();
}

/**
 * @param {{ notes: Record<string, string>, sections: Array<{ type: string, items: Record<string, string[]> }> }} release
 * @param {import("./changelog-locales.mjs").ChangelogLocale} locale
 */
function renderCollapsibleLocale(release, locale) {
  return [
    "<details>",
    `<summary>${LOCALE_SUMMARY_LABELS[locale]}</summary>`,
    "",
    renderLocaleNotes(release, locale),
    "",
    "</details>",
  ].join("\n");
}

/**
 * @param {{ notes: Record<string, string>, sections: Array<{ type: string, items: Record<string, string[]> }> }} release
 * @param {{ mode?: "github" | "all" | import("./changelog-locales.mjs").ChangelogLocale }} [options]
 */
export function renderReleaseNotes(release, options = {}) {
  validateRelease(release);
  const mode = options.mode ?? "github";

  if (mode === "all") {
    const parts = [];
    for (const locale of CHANGELOG_LOCALES) {
      parts.push(`## ${LOCALE_SUMMARY_LABELS[locale]}`, "", renderLocaleNotes(release, locale), "");
    }
    return parts.join("\n").trim();
  }

  if (CHANGELOG_LOCALES.includes(mode)) {
    return renderLocaleNotes(release, mode);
  }

  const parts = [renderLocaleNotes(release, DEFAULT_RELEASE_LOCALE)];
  for (const locale of CHANGELOG_LOCALES) {
    if (locale === DEFAULT_RELEASE_LOCALE) {
      continue;
    }
    parts.push("", renderCollapsibleLocale(release, locale));
  }

  return parts.join("\n").trim();
}

function main() {
  const file = process.argv[2];
  const expectedTag = process.argv[3];
  const localeFlagIndex = process.argv.indexOf("--locale");
  const localeArg = localeFlagIndex >= 0 ? process.argv[localeFlagIndex + 1] : undefined;
  const allLocales = process.argv.includes("--all");

  if (!file) {
    console.error(
      "Usage: node scripts/render-release-notes.mjs <release.json> [expected-tag] [--locale en|zh-CN|ja] [--all]",
    );
    process.exit(1);
  }

  const release = JSON.parse(readFileSync(file, "utf8"));
  if (expectedTag && !expectedTag.startsWith("--") && release.tag !== expectedTag) {
    console.error(`Tag mismatch: expected ${expectedTag}, release.json has ${release.tag}`);
    process.exit(1);
  }

  try {
    const mode = allLocales ? "all" : localeArg ?? "github";
    process.stdout.write(renderReleaseNotes(release, { mode }));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(message);
    process.exit(1);
  }
}

main();
