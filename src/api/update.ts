/**
 * 文件名称：update.ts
 *
 * 文件功能：
 * 封装应用更新相关 Tauri 调用。
 *
 * 主要职责：
 * - 读取当前版本
 * - 检查、下载并安装更新
 * - 重启应用
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-25
 */

import { invoke } from "@/api/invoke";
import type { AppLocale, UpdateInfo } from "@/types";

export const updateApi = {
  getVersion(): Promise<string> {
    return invoke<string>("get_app_version");
  },
  check(locale: AppLocale): Promise<UpdateInfo> {
    return invoke<UpdateInfo>("check_update", { locale });
  },
  downloadAndInstall(): Promise<void> {
    return invoke<void>("download_and_install_update");
  },
  restart(): Promise<void> {
    return invoke<void>("restart_app");
  },
};
