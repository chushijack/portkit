/**
 * 文件名称：settings.ts
 *
 * 文件功能：
 * 管理应用设置、主题与界面语言。
 *
 * 主要职责：
 * - 加载/保存设置
 * - 切换亮暗色并播放过渡动画
 * - 按系统或用户选择应用语言
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { settingsApi } from "@/api";
import { runThemeTransition } from "@/composables/useThemeTransition";
import { applyI18nLocale } from "@/i18n";
import { resolveLocale } from "@/i18n/locale";
import {
  DEFAULT_SETTINGS,
  type AppLocale,
  type AppSettings,
  type LanguageMode,
  type ThemeMode,
} from "@/types";
import { toErrorMessage } from "@/utils/error";
import { isTauriRuntime } from "@/utils/tauri";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
  const loading = ref(false);

  const resolvedTheme = computed<"light" | "dark">(() => {
    if (settings.value.theme === "system") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    return settings.value.theme;
  });

  const resolvedLocale = computed<AppLocale>(() => resolveLocale(settings.value.locale));

  function applyTheme(theme: ThemeMode = settings.value.theme): void {
    const dark =
      theme === "dark" ||
      (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  function applyLocale(mode: LanguageMode = settings.value.locale): void {
    applyI18nLocale(resolveLocale(mode));
  }

  async function setTheme(theme: ThemeMode, origin?: { x: number; y: number }): Promise<void> {
    await runThemeTransition(() => {
      settings.value = { ...settings.value, theme };
      applyTheme(theme);
    }, origin);
    if (!isTauriRuntime()) {
      return;
    }
    try {
      settings.value = await settingsApi.save(settings.value);
      applyTheme(settings.value.theme);
      applyLocale(settings.value.locale);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    }
  }

  async function setLocale(locale: LanguageMode): Promise<void> {
    settings.value = { ...settings.value, locale };
    applyLocale(locale);
    if (!isTauriRuntime()) {
      return;
    }
    try {
      settings.value = await settingsApi.save(settings.value);
      applyLocale(settings.value.locale);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    }
  }

  async function load(): Promise<void> {
    loading.value = true;
    try {
      if (!isTauriRuntime()) {
        settings.value = { ...DEFAULT_SETTINGS };
        applyTheme(settings.value.theme);
        applyLocale(settings.value.locale);
        return;
      }
      settings.value = await settingsApi.get();
      applyTheme(settings.value.theme);
      applyLocale(settings.value.locale);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    } finally {
      loading.value = false;
    }
  }

  async function save(next: AppSettings): Promise<void> {
    const themeChanged = next.theme !== settings.value.theme;
    if (themeChanged) {
      await runThemeTransition(() => {
        settings.value = { ...next };
        applyTheme(next.theme);
        applyLocale(next.locale);
      });
    } else {
      settings.value = { ...next };
      applyLocale(next.locale);
    }
    if (!isTauriRuntime()) {
      return;
    }
    try {
      settings.value = await settingsApi.save(next);
      applyTheme(settings.value.theme);
      applyLocale(settings.value.locale);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    }
  }

  async function patch(partial: Partial<AppSettings>): Promise<void> {
    await save({ ...settings.value, ...partial });
  }

  function bindSystemTheme(): void {
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (settings.value.theme === "system") {
        applyTheme("system");
      }
    });
  }

  function bindSystemLocale(): void {
    window.addEventListener("languagechange", () => {
      if (settings.value.locale === "system") {
        applyLocale("system");
      }
    });
  }

  bindSystemTheme();
  bindSystemLocale();
  applyTheme();
  applyLocale();

  return {
    settings,
    loading,
    resolvedTheme,
    resolvedLocale,
    load,
    save,
    patch,
    applyTheme,
    applyLocale,
    setTheme,
    setLocale,
  };
});
