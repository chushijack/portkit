/**
 * 文件名称：useHotkeys.ts
 *
 * 文件功能：
 * 主窗口本地快捷键，聚焦搜索框。
 *
 * 主要职责：
 * - 监听 Ctrl/Cmd + K
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { onMounted, onUnmounted } from "vue";

export function useHotkeys(onSearch: () => void): void {
  function handler(event: KeyboardEvent): void {
    const modifier = event.metaKey || event.ctrlKey;
    if (modifier && event.key.toLowerCase() === "k") {
      event.preventDefault();
      onSearch();
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handler);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handler);
  });
}
