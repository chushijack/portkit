/**
 * 文件名称：tauri.ts
 *
 * 文件功能：
 * 判断是否在 Tauri WebView 中运行，并安全读取窗口信息。
 *
 * 主要职责：
 * - 避免在纯浏览器调试时调用 Tauri API 崩溃
 * - 提供当前 webview label 的安全读取
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-29
 */

import type { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export type WebviewLabel = "main" | "quick-search";

/** 当前页面是否运行在 Tauri WebView 内。 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 读取当前 webview label；纯浏览器调试时回退为 main。 */
export function getWebviewLabel(): WebviewLabel {
  if (!isTauriRuntime()) {
    return "main";
  }

  try {
    const label = getCurrentWebviewWindow().label;
    return label === "quick-search" ? "quick-search" : "main";
  } catch {
    return "main";
  }
}

/** 返回当前 webview 实例；非 Tauri 环境时为 null。 */
export function getCurrentWebviewWindowOrNull(): WebviewWindow | null {
  if (!isTauriRuntime()) {
    return null;
  }

  try {
    return getCurrentWebviewWindow();
  } catch {
    return null;
  }
}

/** 非 Tauri 环境返回 null，避免 listen 在浏览器调试时崩溃。 */
export async function listenSafe<T>(
  event: string,
  handler: (event: Event<T>) => void,
): Promise<UnlistenFn | null> {
  if (!isTauriRuntime()) {
    return null;
  }
  return listen<T>(event, handler);
}

/**
 * 在 Tauri 环境执行任务；纯浏览器调试时返回 fallback，避免未捕获的 Promise 拒绝。
 */
export async function withTauriRuntime<T>(
  fallback: () => T | Promise<T>,
  task: () => Promise<T>,
): Promise<T> {
  if (!isTauriRuntime()) {
    return fallback();
  }
  return task();
}
