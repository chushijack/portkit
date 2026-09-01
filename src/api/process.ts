/**
 * 文件名称：process.ts
 *
 * 文件功能：
 * 封装进程查询与结束相关 Tauri 调用。
 *
 * 主要职责：
 * - 查询进程详情
 * - 打开进程目录
 * - 结束进程
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import type { KillMode, KillOutcome, ProcessDetail } from "@/types";

export const processApi = {
  getProcessDetail(pid: number): Promise<ProcessDetail> {
    return invoke<ProcessDetail>("get_process_detail", { pid });
  },
  openProcessDirectory(pid: number): Promise<void> {
    return invoke<void>("open_process_directory", { pid });
  },
  killProcess(payload: {
    pid: number;
    port?: number | null;
    protocol?: string | null;
    processName?: string | null;
    mode: KillMode;
  }): Promise<KillOutcome> {
    return invoke<KillOutcome>("kill_process", payload);
  },
};
