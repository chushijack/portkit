<!--
文件名称：AppSidebar.vue

文件功能：
主界面左侧导航，对应原型图侧边栏。

主要职责：
- 展示 PortKit 品牌与页面导航
- 展示实时监控运行状态

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <aside class="sidebar">
    <div class="brand">
      <img class="brand-logo" src="/logo.png" alt="PortKit" width="32" height="32" />
      <span>PortKit</span>
    </div>

    <nav class="nav">
      <RouterLink
        v-for="item in items"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
      >
        <FaIcon :icon="item.icon" :size="16" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>

    <div class="monitor-card" :class="{ off: !monitorStore.running }">
      <div class="monitor-title">
        <span class="dot" />
        {{ monitorStore.running ? $t("sidebar.monitorOn") : $t("sidebar.monitorOff") }}
      </div>
      <div class="monitor-time">{{ $t("sidebar.elapsed", { time: monitorStore.elapsedLabel }) }}</div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import {
  faClock,
  faGear,
  faHeartPulse,
  faStar,
  faTableCellsLarge,
} from "@fortawesome/free-solid-svg-icons";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import FaIcon from "@/components/FaIcon.vue";
import { useMonitorStore } from "@/stores/monitor";

const route = useRoute();
const monitorStore = useMonitorStore();
const { t } = useI18n();

const items = computed(() => [
  { path: "/", label: t("nav.ports"), icon: faTableCellsLarge },
  { path: "/monitor", label: t("nav.monitor"), icon: faHeartPulse },
  { path: "/favorites", label: t("nav.favorites"), icon: faStar },
  { path: "/history", label: t("nav.history"), icon: faClock },
  { path: "/settings", label: t("nav.settings"), icon: faGear },
]);

function isActive(path: string): boolean {
  if (path === "/") {
    return route.path === "/" || route.path.startsWith("/process");
  }
  return route.path.startsWith(path);
}
</script>

<style scoped>
.sidebar {
  width: var(--pk-sidebar-width);
  height: 100%;
  background: var(--pk-sidebar);
  border-right: 1px solid var(--pk-border);
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 20px;
  font-weight: 700;
  padding: 4px 8px 24px;
}

.brand-logo {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  object-fit: contain;
  flex-shrink: 0;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 42px;
  padding: 0 12px;
  border-radius: 10px;
  color: var(--pk-text-secondary);
  text-decoration: none;
  font-weight: 500;
}

.nav-item.active,
.nav-item:hover {
  background: var(--pk-primary-soft);
  color: var(--pk-primary);
}

.monitor-card {
  border-radius: 12px;
  padding: 14px;
  background: var(--pk-success-soft);
  color: var(--pk-monitor-on);
}

.monitor-card.off {
  background: var(--pk-muted-bg);
  color: var(--pk-text-secondary);
}

.monitor-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.monitor-time {
  margin-top: 6px;
  font-size: 12px;
}
</style>
