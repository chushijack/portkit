/**
 * 文件名称：favorite.ts
 *
 * 文件功能：
 * 管理收藏端口列表。
 *
 * 主要职责：
 * - 加载收藏
 * - 按端口唯一新增
 * - 更新备注与取消收藏
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { ref } from "vue";
import { favoriteApi } from "@/api";
import type { FavoritePort } from "@/types";
import { withTauriRuntime, isTauriRuntime } from "@/utils/tauri";

export const useFavoriteStore = defineStore("favorite", () => {
  const items = ref<FavoritePort[]>([]);

  async function load(): Promise<void> {
    items.value = await withTauriRuntime(
      () => [],
      () => favoriteApi.list(),
    );
  }

  async function add(port: number, protocol: string, note: string): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    items.value = await favoriteApi.add(port, protocol, note);
  }

  async function updateNote(port: number, note: string): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    items.value = await favoriteApi.updateNote(port, note);
  }

  async function remove(port: number): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    items.value = await favoriteApi.remove(port);
  }

  /** 同一端口只能有一条收藏，协议不参与去重。 */
  function isFavorite(port: number): boolean {
    return items.value.some((item) => item.port === port);
  }

  return { items, load, add, updateNote, remove, isFavorite };
});
