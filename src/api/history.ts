/**
 * 文件名称：history.ts
 *
 * 文件功能：
 * 封装操作历史相关 Tauri 调用。
 *
 * 主要职责：
 * - 列出历史
 * - 清空历史
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import type { HistoryRecord } from "@/types";

export const historyApi = {
  list(): Promise<HistoryRecord[]> {
    return invoke<HistoryRecord[]>("list_history");
  },
  clear(): Promise<void> {
    return invoke<void>("clear_history");
  },
};
