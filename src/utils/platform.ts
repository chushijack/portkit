/**
 * 文件名称：platform.ts
 *
 * 文件功能：
 * 判断当前运行平台，用于快捷键文案。
 *
 * 主要职责：
 * - 识别 macOS
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export function isMacPlatform(): boolean {
  return navigator.userAgent.toLowerCase().includes("mac");
}
