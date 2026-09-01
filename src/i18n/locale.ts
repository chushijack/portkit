/**
 * 文件名称：locale.ts
 *
 * 文件功能：
 * 解析界面语言：跟随系统或用户指定。
 *
 * 主要职责：
 * - 读取 navigator.language
 * - 映射为中文、英文或日文
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import type { AppLocale, LanguageMode } from "@/types";

const SUPPORTED: AppLocale[] = ["zh-CN", "en", "ja"];

/** 把浏览器/系统语言标签映射为应用支持的语言。 */
export function detectSystemLocale(): AppLocale {
  const tag = (navigator.language || "en").toLowerCase().replace("_", "-");
  if (tag.startsWith("zh")) {
    return "zh-CN";
  }
  if (tag.startsWith("ja")) {
    return "ja";
  }
  if (tag.startsWith("en")) {
    return "en";
  }
  return "en";
}

/** 设置项为跟随系统时读取 OS 语言，否则使用指定语言。 */
export function resolveLocale(mode: LanguageMode): AppLocale {
  if (mode === "system") {
    return detectSystemLocale();
  }
  return SUPPORTED.includes(mode) ? mode : "en";
}
