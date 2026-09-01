<!--
文件名称：Monitor.vue

文件功能：
实时监控页面，展示端口新增与关闭事件。

主要职责：
- 开关监控
- 展示变更时间线

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page">
    <div class="pk-page-header">
      <h1 class="pk-page-title">{{ $t("monitor.title") }}</h1>
      <a-button :type="monitorStore.running ? 'primary' : 'default'" @click="toggle">
        {{ monitorStore.running ? $t("monitor.stop") : $t("monitor.start") }}
      </a-button>
    </div>

    <section class="status pk-card">
      <div>
        <div class="pk-muted">{{ $t("monitor.status") }}</div>
        <strong>{{ monitorStore.running ? $t("monitor.running") : $t("monitor.off") }}</strong>
      </div>
      <div>
        <div class="pk-muted">{{ $t("monitor.elapsed") }}</div>
        <strong>{{ monitorStore.elapsedLabel }}</strong>
      </div>
      <div>
        <div class="pk-muted">{{ $t("monitor.interval") }}</div>
        <strong>{{ $t("monitor.intervalValue", { n: settingsStore.settings.refreshInterval }) }}</strong>
      </div>
    </section>

    <section class="timeline pk-card">
      <a-empty v-if="monitorStore.events.length === 0" :description="$t('monitor.empty')" />
      <a-timeline v-else>
        <a-timeline-item
          v-for="item in monitorStore.events"
          :key="item.id"
          :color="item.type === 'appeared' ? 'green' : 'red'"
        >
          <div class="event">
            <b>{{ item.type === "appeared" ? $t("monitor.appeared") : $t("monitor.closed") }}</b>
            <span>{{ item.time }}</span>
          </div>
          <div class="pk-muted">
            {{ item.protocol }} {{ item.port }} · {{ item.processName }} · PID {{ item.pid }}
          </div>
        </a-timeline-item>
      </a-timeline>
    </section>
  </div>
</template>

<script setup lang="ts">
import { scanPorts } from "@/composables/useAutoRefresh";
import { useMonitorStore } from "@/stores/monitor";
import { useSettingsStore } from "@/stores/settings";

const monitorStore = useMonitorStore();
const settingsStore = useSettingsStore();

function toggle(): void {
  if (monitorStore.running) {
    monitorStore.stop();
    return;
  }
  monitorStore.start();
  void scanPorts({ recordHistory: false });
}
</script>

<style scoped>
.status {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  padding: 20px;
}

.timeline {
  flex: 1;
  overflow: auto;
  padding: 20px;
}

.event {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
</style>
