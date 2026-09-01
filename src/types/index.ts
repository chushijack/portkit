/**
 * 文件名称：index.ts
 *
 * 文件功能：
 * 统一导出前端类型。
 *
 * 主要职责：
 * - 方便页面按需引用业务类型
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export type { CommonPort, PortInfo, ScanMode, ScanRequest } from "./port";
export type { ProcessDetail, KillMode, KillOutcome } from "./process";
export type { FavoritePort } from "./favorite";
export type { HistoryRecord } from "./history";
export type { AppSettings, ThemeMode, AppLocale, LanguageMode } from "./settings";
export { DEFAULT_SETTINGS, isThemeMode, isLanguageMode } from "./settings";
export type { MonitorEvent, MonitorChangeType } from "./monitor";
export type { UpdateInfo, UpdateProgress, UpdateStatus } from "./update";
export type {
  AppRelease,
  ChangelogLocale,
  ChangelogSection,
  ChangelogSectionType,
  LocalizedItems,
  LocalizedNote,
  ResolvedAppRelease,
  ResolvedChangelogSection,
} from "./release";
export { CHANGELOG_LOCALES, resolveAppRelease } from "./release";
