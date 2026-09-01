/**
 * 文件名称：invoke.ts
 *
 * 文件功能：
 * 封装 Tauri invoke 调用，避免纯浏览器调试时崩溃。
 *
 * 主要职责：
 * - 在非 Tauri 环境拒绝调用并返回明确错误
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-29
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "@/utils/tauri";

/** 当前页面不在 Tauri WebView 中运行。 */
export class TauriUnavailableError extends Error {
  constructor() {
    super("Tauri runtime is not available");
    this.name = "TauriUnavailableError";
  }
}

/** 判断错误是否由非 Tauri 环境触发。 */
export function isTauriUnavailableError(error: unknown): boolean {
  return error instanceof TauriUnavailableError;
}

/** 安全调用 Tauri command；纯浏览器环境会拒绝执行。 */
export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    return Promise.reject(new TauriUnavailableError());
  }
  return tauriInvoke<T>(command, args);
}
