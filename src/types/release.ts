/**
 * 文件名称：release.ts
 *
 * 文件功能：
 * 定义 GitHub Release 与更新日志类型。
 *
 * 主要职责：
 * - 约束更新日志页面数据结构
 * - 支持 zh-CN / en / ja 三语内容
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-26
 */

export type ChangelogLocale = "zh-CN" | "en" | "ja";

export const CHANGELOG_LOCALES: readonly ChangelogLocale[] = ["zh-CN", "en", "ja"];

export type ChangelogSectionType = "added" | "improved" | "fixed" | "other";

export interface LocalizedNote {
  "zh-CN": string;
  en: string;
  ja: string;
}

export interface LocalizedItems {
  "zh-CN": string[];
  en: string[];
  ja: string[];
}

export interface ChangelogSection {
  type: ChangelogSectionType;
  items: LocalizedItems;
}

export interface AppRelease {
  tag: string;
  version: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  notes: LocalizedNote;
  sections: ChangelogSection[];
}

export interface ResolvedChangelogSection {
  type: ChangelogSectionType;
  items: string[];
}

export interface ResolvedAppRelease {
  tag: string;
  version: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  notes: string;
  sections: ResolvedChangelogSection[];
}

/**
 * 将多语言 changelog 解析为指定语言的展示结构。
 */
export function resolveAppRelease(release: AppRelease, locale: ChangelogLocale): ResolvedAppRelease {
  const pickItems = (items: LocalizedItems): string[] => items[locale];

  return {
    tag: release.tag,
    version: release.version,
    name: release.name,
    publishedAt: release.publishedAt,
    htmlUrl: release.htmlUrl,
    notes: release.notes[locale],
    sections: release.sections.map((section) => ({
      type: section.type,
      items: pickItems(section.items),
    })),
  };
}
