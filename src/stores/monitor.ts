/**
 * 文件名称：monitor.ts
 *
 * 文件功能：
 * 管理实时监控开关、运行时长与端口变更事件。
 *
 * 主要职责：
 * - 启动/停止监控
 * - 对比两次扫描结果生成提醒
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { i18n } from "@/i18n";
import type { MonitorEvent, PortInfo } from "@/types";
import { portKey } from "@/utils/format";

export const useMonitorStore = defineStore("monitor", () => {
  const running = ref(false);
  const startedAt = ref<number | null>(null);
  const elapsedSeconds = ref(0);
  const events = ref<MonitorEvent[]>([]);
  let timer: number | null = null;

  const elapsedLabel = computed(() => {
    const hours = Math.floor(elapsedSeconds.value / 3600);
    const minutes = Math.floor((elapsedSeconds.value % 3600) / 60);
    const seconds = elapsedSeconds.value % 60;
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  });

  function start(): void {
    if (running.value) {
      return;
    }
    running.value = true;
    startedAt.value = Date.now();
    elapsedSeconds.value = 0;
    timer = window.setInterval(() => {
      if (startedAt.value) {
        elapsedSeconds.value = Math.floor((Date.now() - startedAt.value) / 1000);
      }
    }, 1000);
  }

  function stop(): void {
    running.value = false;
    startedAt.value = null;
    elapsedSeconds.value = 0;
    if (timer !== null) {
      window.clearInterval(timer);
      timer = null;
    }
  }

  function diff(previous: PortInfo[], current: PortInfo[]): MonitorEvent[] {
    const prevMap = new Map(previous.map((item) => [portKey(item), item]));
    const nextMap = new Map(current.map((item) => [portKey(item), item]));
    const changes: MonitorEvent[] = [];
    const locale = i18n.global.locale.value;
    const tag = locale === "zh-CN" ? "zh-CN" : locale === "ja" ? "ja-JP" : "en-US";
    const now = new Date().toLocaleTimeString(tag, { hour12: false });

    for (const [key, item] of nextMap) {
      if (!prevMap.has(key)) {
        changes.push({
          id: `${key}-in-${Date.now()}`,
          time: now,
          type: "appeared",
          port: item.port,
          protocol: item.protocol,
          processName: item.processName,
          pid: item.pid,
        });
      }
    }
    for (const [key, item] of prevMap) {
      if (!nextMap.has(key)) {
        changes.push({
          id: `${key}-out-${Date.now()}`,
          time: now,
          type: "closed",
          port: item.port,
          protocol: item.protocol,
          processName: item.processName,
          pid: item.pid,
        });
      }
    }

    if (changes.length > 0) {
      events.value = [...changes, ...events.value].slice(0, 200);
    }
    return changes;
  }

  return { running, elapsedSeconds, elapsedLabel, events, start, stop, diff };
});
