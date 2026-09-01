/**
 * 文件名称：favorite.ts
 *
 * 文件功能：
 * 封装收藏端口相关 Tauri 调用。
 *
 * 主要职责：
 * - 列出、新增、更新备注、删除收藏
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import type { FavoritePort } from "@/types";

export const favoriteApi = {
  list(): Promise<FavoritePort[]> {
    return invoke<FavoritePort[]>("list_favorites");
  },
  add(port: number, protocol: string, note: string): Promise<FavoritePort[]> {
    return invoke<FavoritePort[]>("add_favorite", { port, protocol, note });
  },
  updateNote(port: number, note: string): Promise<FavoritePort[]> {
    return invoke<FavoritePort[]>("update_favorite_note", { port, note });
  },
  remove(port: number): Promise<FavoritePort[]> {
    return invoke<FavoritePort[]>("remove_favorite", { port });
  },
};
