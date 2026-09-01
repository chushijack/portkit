/**
 * 文件名称：browserDev.ts
 *
 * 文件功能：
 * 纯浏览器预览模式下的全局保护。
 *
 * 主要职责：
 * - 吞掉遗漏的 Tauri 不可用 Promise 拒绝，避免控制台噪音
 * - 提示开发者应使用 Tauri 桌面窗口验证完整功能
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-29
 */

import { isTauriUnavailableError } from "@/api/invoke";
import { isTauriRuntime } from "@/utils/tauri";

let warned = false;

/** 在应用启动时注册浏览器预览保护。 */
export function setupBrowserDevGuards(): void {
  if (isTauriRuntime()) {
    return;
  }

  window.addEventListener("unhandledrejection", (event) => {
    if (isTauriUnavailableError(event.reason)) {
      event.preventDefault();
    }
  });

  if (!warned) {
    warned = true;
    console.info(
      "[PortKit] 当前为浏览器预览模式，仅用于 UI 调试。请使用 `pnpm tauri dev` 弹出的桌面窗口验证端口扫描等完整功能。",
    );
  }
}
