/**
 * 文件名称：update.ts
 *
 * 文件功能：
 * 管理应用更新状态与弹窗展示。
 *
 * 主要职责：
 * - 读取当前版本
 * - 检查更新并缓存结果
 * - 跟踪下载进度
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-25
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { updateApi } from "@/api";
import { i18n } from "@/i18n";
import type { AppLocale, UpdateInfo, UpdateProgress, UpdateStatus } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { isTauriRuntime } from "@/utils/tauri";
import packageJson from "../../package.json";

export const useUpdateStore = defineStore("update", () => {
  const currentVersion = ref("");
  const info = ref<UpdateInfo | null>(null);
  const status = ref<UpdateStatus>("idle");
  const dialogOpen = ref(false);
  const downloadedBytes = ref(0);
  const totalBytes = ref<number | null>(null);
  const listeners: UnlistenFn[] = [];

  const progressPercent = computed(() => {
    if (!totalBytes.value || totalBytes.value <= 0) {
      return 0;
    }
    return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100));
  });

  const busy = computed(
    () => status.value === "checking" || status.value === "downloading" || status.value === "installing",
  );

  async function bindProgress(): Promise<void> {
    if (!isTauriRuntime() || listeners.length > 0) {
      return;
    }
    listeners.push(
      await listen<UpdateProgress>("update-download-progress", (event) => {
        downloadedBytes.value = event.payload.downloaded;
        totalBytes.value = event.payload.total;
        status.value = "downloading";
      }),
    );
    listeners.push(
      await listen("update-download-finished", () => {
        status.value = "installing";
      }),
    );
  }

  async function loadVersion(): Promise<void> {
    if (!isTauriRuntime()) {
      currentVersion.value = packageJson.version;
      return;
    }
    currentVersion.value = await updateApi.getVersion();
  }

  async function check(): Promise<UpdateInfo> {
    if (!isTauriRuntime()) {
      const result: UpdateInfo = {
        available: false,
        currentVersion: packageJson.version,
        latestVersion: packageJson.version,
        notes: "",
        date: null,
      };
      info.value = result;
      currentVersion.value = result.currentVersion;
      status.value = "upToDate";
      return result;
    }
    await bindProgress();
    status.value = "checking";
    try {
      const result = await updateApi.check(i18n.global.locale.value as AppLocale);
      info.value = result;
      currentVersion.value = result.currentVersion;
      status.value = result.available ? "available" : "upToDate";
      return result;
    } catch (error) {
      status.value = "error";
      throw new Error(toErrorMessage(error));
    }
  }

  async function install(): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }
    await bindProgress();
    downloadedBytes.value = 0;
    totalBytes.value = null;
    status.value = "downloading";
    try {
      await updateApi.downloadAndInstall();
      status.value = "installing";
    } catch (error) {
      status.value = "error";
      throw new Error(toErrorMessage(error));
    }
  }

  function openDialog(): void {
    dialogOpen.value = true;
  }

  function closeDialog(): void {
    dialogOpen.value = false;
  }

  function moveToBackground(): void {
    dialogOpen.value = false;
  }

  const backgroundUpdating = computed(
    () => !dialogOpen.value && (status.value === "downloading" || status.value === "installing"),
  );

  function dispose(): void {
    listeners.splice(0).forEach((unlisten) => unlisten());
  }

  return {
    currentVersion,
    info,
    status,
    dialogOpen,
    downloadedBytes,
    totalBytes,
    progressPercent,
    busy,
    backgroundUpdating,
    loadVersion,
    check,
    install,
    openDialog,
    closeDialog,
    moveToBackground,
    dispose,
  };
});
