/**
 * 文件名称：process.ts
 *
 * 文件功能：
 * 管理进程详情与结束进程确认状态。
 *
 * 主要职责：
 * - 加载进程详情
 * - 打开目录、结束进程
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { ref } from "vue";
import { processApi } from "@/api";
import type { KillMode, KillOutcome, PortInfo, ProcessDetail } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { isTauriRuntime } from "@/utils/tauri";

export const useProcessStore = defineStore("process", () => {
  const detail = ref<ProcessDetail | null>(null);
  const loading = ref(false);

  async function load(pid: number): Promise<void> {
    loading.value = true;
    try {
      if (!isTauriRuntime()) {
        detail.value = null;
        return;
      }
      detail.value = await processApi.getProcessDetail(pid);
    } catch (error) {
      detail.value = null;
      throw new Error(toErrorMessage(error));
    } finally {
      loading.value = false;
    }
  }

  async function openDirectory(pid: number): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    await processApi.openProcessDirectory(pid);
  }

  async function kill(payload: {
    pid: number;
    port?: number | null;
    protocol?: string | null;
    processName?: string | null;
    mode: KillMode;
  }): Promise<KillOutcome> {
    if (!isTauriRuntime()) {
      throw new Error("Tauri runtime is not available");
    }
    return processApi.killProcess(payload);
  }

  function fromPort(port: PortInfo): { pid: number; port: number; protocol: string; processName: string } {
    return {
      pid: port.pid,
      port: port.port,
      protocol: port.protocol,
      processName: port.processName,
    };
  }

  return { detail, loading, load, openDirectory, kill, fromPort };
});
