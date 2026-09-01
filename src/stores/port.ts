/**
 * 文件名称：port.ts
 *
 * 文件功能：
 * 管理端口列表、搜索过滤与统计数据。
 *
 * 主要职责：
 * - 触发扫描
 * - 按关键字过滤
 * - 计算统计卡片数据
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { portApi } from "@/api";
import { useSettingsStore } from "@/stores/settings";
import type { CommonPort, PortInfo, ScanMode, ScanRequest } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { listenHost, portKey } from "@/utils/format";
import { withTauriRuntime } from "@/utils/tauri";

export const usePortStore = defineStore("port", () => {
  const ports = ref<PortInfo[]>([]);
  const keyword = ref("");
  const loading = ref(false);
  const lastError = ref("");
  const commonPorts = ref<CommonPort[]>([
    { port: 3000, label: "React / Next.js" },
    { port: 5173, label: "Vite" },
    { port: 8080, label: "Spring / Go / Vue" },
  ]);

  const filteredPorts = computed(() => {
    const query = keyword.value.trim().toLowerCase();
    if (!query) {
      return ports.value;
    }
    return ports.value.filter((item) => {
      return (
        String(item.port).includes(query) ||
        String(item.pid).includes(query) ||
        item.processName.toLowerCase().includes(query) ||
        (item.serviceLabel ?? "").toLowerCase().includes(query)
      );
    });
  });

  const stats = computed(() => {
    const tcpCount = ports.value.filter((item) => item.protocol === "TCP").length;
    const udpCount = ports.value.filter((item) => item.protocol === "UDP").length;
    const listenAddresses = new Set(
      ports.value.map((item) => listenHost(item.listenAddress)),
    );
    return {
      total: ports.value.length,
      tcp: tcpCount,
      udp: udpCount,
      addresses: listenAddresses.size,
    };
  });

  function buildRequest(overrides: Partial<ScanRequest> = {}): ScanRequest {
    const settingsStore = useSettingsStore();
    const mode = (overrides.mode ?? settingsStore.settings.scanMode) as ScanMode;
    return {
      mode,
      startPort: overrides.startPort ?? settingsStore.settings.customStart,
      endPort: overrides.endPort ?? settingsStore.settings.customEnd,
      recordHistory: overrides.recordHistory ?? false,
    };
  }

  async function scan(overrides: Partial<ScanRequest> = {}): Promise<PortInfo[]> {
    loading.value = true;
    lastError.value = "";
    try {
      const result = await withTauriRuntime(
        () => [] as PortInfo[],
        () => portApi.scanPorts(buildRequest(overrides)),
      );
      ports.value = result;
      return result;
    } catch (error) {
      lastError.value = toErrorMessage(error);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  async function loadCommonPorts(): Promise<void> {
    commonPorts.value = await withTauriRuntime(
      () => commonPorts.value,
      () => portApi.getCommonPorts(),
    );
  }

  function findByKey(key: string): PortInfo | undefined {
    return ports.value.find((item) => portKey(item) === key);
  }

  return {
    ports,
    keyword,
    loading,
    lastError,
    commonPorts,
    filteredPorts,
    stats,
    scan,
    loadCommonPorts,
    findByKey,
  };
});
