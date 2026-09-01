/**
 * 文件名称：release.ts
 *
 * 文件功能：
 * 封装更新日志相关 Tauri 调用。
 *
 * 主要职责：
 * - 拉取 GitHub Release 列表
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-26
 */

import { invoke } from "@/api/invoke";
import type { AppLocale, AppRelease, ResolvedAppRelease } from "@/types";

export const releaseApi = {
  list(): Promise<AppRelease[]> {
    return invoke<AppRelease[]>("get_releases");
  },
  listResolved(locale: AppLocale): Promise<ResolvedAppRelease[]> {
    return invoke<ResolvedAppRelease[]>("get_resolved_releases", { locale });
  },
};
