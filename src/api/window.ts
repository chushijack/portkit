/**
 * 文件名称：window.ts
 *
 * 文件功能：
 * 封装窗口控制相关 Tauri 调用。
 *
 * 主要职责：
 * - 隐藏快速搜索窗口
 * - 打开主窗口并跳转路由
 * - 用系统浏览器打开 GitHub 仓库与官网更新日志
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import { openUrl } from "@tauri-apps/plugin-opener";
import { isTauriRuntime } from "@/utils/tauri";

const GITHUB_REPO_URL = "https://github.com/chushijack/portkit";
const CHANGELOG_URL = "https://portkit.chushijack.com/changelog";

async function openExternal(url: string): Promise<void> {
  if (!isTauriRuntime()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }
  await openUrl(url);
}

export const windowApi = {
  hideQuickSearch(): Promise<void> {
    if (!isTauriRuntime()) {
      return Promise.resolve();
    }
    return invoke<void>("hide_quick_search");
  },
  openMainRoute(_route: string): Promise<void> {
    if (!isTauriRuntime()) {
      return Promise.resolve();
    }
    return invoke<void>("open_main_route", { route: _route });
  },
  openGithub(): Promise<void> {
    return openExternal(GITHUB_REPO_URL);
  },
  openChangelog(): Promise<void> {
    return openExternal(CHANGELOG_URL);
  },
};
