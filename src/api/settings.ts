/**
 * 文件名称：settings.ts
 *
 * 文件功能：
 * 封装应用设置与开机启动相关调用。
 *
 * 主要职责：
 * - 读写设置
 * - 同步开机启动插件状态
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import type { AppSettings } from "@/types";
import { isTauriRuntime } from "@/utils/tauri";

export const settingsApi = {
  get(): Promise<AppSettings> {
    return invoke<AppSettings>("get_settings");
  },
  async save(settings: AppSettings): Promise<AppSettings> {
    if (!isTauriRuntime()) {
      return settings;
    }
    const saved = await invoke<AppSettings>("save_settings", { settings });
    const currentlyEnabled = await isEnabled();
    if (saved.autostart && !currentlyEnabled) {
      await enable();
    }
    if (!saved.autostart && currentlyEnabled) {
      await disable();
    }
    return saved;
  },
};
