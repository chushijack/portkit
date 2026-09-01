/**
 * 文件名称：clipboard.ts
 *
 * 文件功能：
 * 跨环境写入剪贴板。
 *
 * 主要职责：
 * - Tauri 环境使用系统剪贴板插件
 * - 浏览器预览环境回退到 navigator.clipboard
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-29
 */

import { isTauriRuntime } from "@/utils/tauri";

/** 将文本写入剪贴板。 */
export async function writeClipboard(text: string): Promise<void> {
  if (isTauriRuntime()) {
    const { writeText } = await import("@tauri-apps/plugin-clipboard-manager");
    await writeText(text);
    return;
  }
  await navigator.clipboard.writeText(text);
}
