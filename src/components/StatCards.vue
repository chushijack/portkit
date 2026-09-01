<!--
文件名称：StatCards.vue

文件功能：
端口列表页顶部统计卡片。

主要职责：
- 展示全部、TCP、UDP 与监听地址数量

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="stats">
    <article v-for="item in cards" :key="item.label" class="stat pk-card">
      <div class="icon" :style="{ background: item.soft, color: item.color }">
        <FaIcon :icon="item.icon" :size="18" />
      </div>
      <div>
        <div class="label">{{ item.label }}</div>
        <div class="value">{{ item.value }}</div>
      </div>
    </article>
  </div>
</template>

<script setup lang="ts">
import {
  faGlobe,
  faNetworkWired,
  faServer,
  faTowerBroadcast,
} from "@fortawesome/free-solid-svg-icons";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import FaIcon from "@/components/FaIcon.vue";
import { usePortStore } from "@/stores/port";

const portStore = usePortStore();
const { t } = useI18n();

const cards = computed(() => [
  {
    label: t("stats.total"),
    value: portStore.stats.total,
    icon: faServer,
    color: "#3d7eff",
    soft: "var(--pk-stat-blue-soft)",
  },
  {
    label: t("stats.tcp"),
    value: portStore.stats.tcp,
    icon: faNetworkWired,
    color: "#22c55e",
    soft: "var(--pk-stat-green-soft)",
  },
  {
    label: t("stats.udp"),
    value: portStore.stats.udp,
    icon: faTowerBroadcast,
    color: "#8b5cf6",
    soft: "var(--pk-stat-purple-soft)",
  },
  {
    label: t("stats.addresses"),
    value: portStore.stats.addresses,
    icon: faGlobe,
    color: "#f59e0b",
    soft: "var(--pk-stat-orange-soft)",
  },
]);
</script>

<style scoped>
.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
}

.icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.label {
  color: var(--pk-text-secondary);
  font-size: 13px;
}

.value {
  font-size: 28px;
  font-weight: 700;
  line-height: 1.2;
}
</style>
