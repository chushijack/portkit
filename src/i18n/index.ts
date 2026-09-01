/**
 * 文件名称：index.ts
 *
 * 文件功能：
 * 创建 vue-i18n 实例，启动时先按系统语言显示。
 *
 * 主要职责：
 * - 注册中/英/日文案
 * - 设置加载后再按用户偏好覆盖
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { createI18n } from "vue-i18n";
import { detectSystemLocale } from "./locale";
import en from "./locales/en";
import ja from "./locales/ja";
import zhCN from "./locales/zh-CN";
import type { AppLocale } from "@/types";

const initialLocale = detectSystemLocale();

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: initialLocale,
  fallbackLocale: "en",
  messages: {
    "zh-CN": zhCN,
    en,
    ja,
  },
});

/** 给 html lang 和 vue-i18n 同步已解析语言。 */
export function applyI18nLocale(locale: AppLocale): void {
  i18n.global.locale.value = locale;
  document.documentElement.lang = locale;
}
