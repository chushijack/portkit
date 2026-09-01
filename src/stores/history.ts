/**
 * 文件名称：history.ts
 *
 * 文件功能：
 * 管理操作历史列表。
 *
 * 主要职责：
 * - 加载历史
 * - 清空历史
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { ref } from "vue";
import { historyApi } from "@/api";
import type { HistoryRecord } from "@/types";
import { isTauriRuntime, withTauriRuntime } from "@/utils/tauri";

export const useHistoryStore = defineStore("history", () => {
  const items = ref<HistoryRecord[]>([]);

  async function load(): Promise<void> {
    items.value = await withTauriRuntime(
      () => [],
      () => historyApi.list(),
    );
  }

  async function clear(): Promise<void> {
    if (!isTauriRuntime()) {
      items.value = [];
      return;
    }
    await historyApi.clear();
    items.value = [];
  }

  return { items, load, clear };
});
