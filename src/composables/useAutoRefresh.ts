/**
 * 文件名称：useAutoRefresh.ts
 *
 * 文件功能：
 * 按设置间隔自动扫描端口，并在监控开启时对比变化。
 *
 * 主要职责：
 * - 手动扫描
 * - 定时刷新
 * - 触发端口变化通知
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { sendNotification } from "@tauri-apps/plugin-notification";
import { watch } from "vue";
import { i18n } from "@/i18n";
import { useMonitorStore } from "@/stores/monitor";
import { usePortStore } from "@/stores/port";
import { useSettingsStore } from "@/stores/settings";
import type { MonitorEvent, PortInfo, ScanRequest } from "@/types";
import { isTauriRuntime } from "@/utils/tauri";

let timer: number | null = null;

async function notifyChanges(changes: MonitorEvent[]): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  const settingsStore = useSettingsStore();
  if (!settingsStore.settings.notifications || changes.length === 0) {
    return;
  }
  const appeared = changes.filter((item) => item.type === "appeared");
  const closed = changes.filter((item) => item.type === "closed");
  const t = i18n.global.t;
  const parts: string[] = [];
  if (appeared.length > 0) {
    parts.push(t("monitor.notifyAppeared", { ports: appeared.map((item) => item.port).join(", ") }));
  }
  if (closed.length > 0) {
    parts.push(t("monitor.notifyClosed", { ports: closed.map((item) => item.port).join(", ") }));
  }
  try {
    await sendNotification({ title: t("monitor.notifyTitle"), body: parts.join(" · ") });
  } catch {
    // 未授权通知时不影响扫描
  }
}

/** 扫描端口；监控开启时会对比变化并发送通知。 */
export async function scanPorts(overrides: Partial<ScanRequest> = {}): Promise<PortInfo[]> {
  if (!isTauriRuntime()) {
    return [];
  }
  const portStore = usePortStore();
  const monitorStore = useMonitorStore();
  const previous = [...portStore.ports];
  const current = await portStore.scan(overrides);
  if (monitorStore.running) {
    const changes = monitorStore.diff(previous, current);
    await notifyChanges(changes);
  }
  return current;
}

function stopTimer(): void {
  if (timer !== null) {
    window.clearInterval(timer);
    timer = null;
  }
}

function startTimer(): void {
  const monitorStore = useMonitorStore();
  const settingsStore = useSettingsStore();
  stopTimer();
  if (!monitorStore.running) {
    return;
  }
  const interval = Math.max(1, settingsStore.settings.refreshInterval) * 1000;
  timer = window.setInterval(() => {
    void scanPorts({ recordHistory: false });
  }, interval);
}

/** 仅在主窗口根组件调用一次，避免重复定时器。 */
export function useAutoRefresh(): { scan: typeof scanPorts } {
  const monitorStore = useMonitorStore();
  const settingsStore = useSettingsStore();
  watch(
    () => [monitorStore.running, settingsStore.settings.refreshInterval] as const,
    () => {
      startTimer();
    },
  );
  return { scan: scanPorts };
}
