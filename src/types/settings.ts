/**
 * 文件名称：settings.ts
 *
 * 文件功能：
 * 定义应用设置数据结构。
 *
 * 主要职责：
 * - 约束扫描、系统、外观、语言与更新设置字段
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import type { ScanMode } from "./port";

export type ThemeMode = "light" | "dark" | "system";

/** 用户可选语言；system 表示跟随操作系统。 */
export type AppLocale = "zh-CN" | "en" | "ja";

export type LanguageMode = AppLocale | "system";

/** 校验主题单选项，避免把非法值写入设置。 */
export function isThemeMode(value: string): value is ThemeMode {
  return value === "light" || value === "dark" || value === "system";
}

/** 校验语言单选项，避免把非法值写入设置。 */
export function isLanguageMode(value: string): value is LanguageMode {
  return value === "system" || value === "zh-CN" || value === "en" || value === "ja";
}

export interface AppSettings {
  scanMode: ScanMode;
  customStart: number;
  customEnd: number;
  refreshInterval: number;
  autoScanOnStartup: boolean;
  autostart: boolean;
  backgroundRun: boolean;
  notifications: boolean;
  theme: ThemeMode;
  locale: LanguageMode;
  autoCheckUpdate: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  scanMode: "all",
  customStart: 3000,
  customEnd: 9000,
  refreshInterval: 3,
  autoScanOnStartup: true,
  autostart: false,
  backgroundRun: true,
  notifications: true,
  theme: "system",
  locale: "system",
  autoCheckUpdate: true,
};
