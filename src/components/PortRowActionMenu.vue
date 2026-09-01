<!--
文件名称：PortRowActionMenu.vue

文件功能：
端口行操作菜单，供行内更多按钮与右键菜单共用。

主要职责：
- 展示收藏、复制 PID、优雅关闭与强制结束
- 向父组件回传当前行操作

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-menu :selectable="false">
    <a-menu-item key="favorite" @click="emit('favorite', record)">
      <FaIcon :icon="faStar" :size="13" />
      {{ favoriteStore.isFavorite(record.port) ? $t("port.unfavorite") : $t("port.favorite") }}
    </a-menu-item>
    <a-menu-item key="copy" @click="emit('copy', record)">
      <FaIcon :icon="faCopy" :size="13" />
      {{ $t("port.copyPid") }}
    </a-menu-item>
    <a-menu-divider />
    <a-menu-item key="killGraceful" @click="emit('kill', record, 'graceful')">
      <FaIcon :icon="faHand" :size="13" />
      {{ $t("port.killGraceful") }}
    </a-menu-item>
    <a-menu-item key="killForce" danger @click="emit('kill', record, 'force')">
      <FaIcon :icon="faBolt" :size="13" />
      {{ $t("port.killForce") }}
    </a-menu-item>
  </a-menu>
</template>

<script setup lang="ts">
import { faBolt, faCopy, faHand, faStar } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "@/components/FaIcon.vue";
import { useFavoriteStore } from "@/stores/favorite";
import type { KillMode, PortInfo } from "@/types";

defineProps<{
  record: PortInfo;
}>();

const emit = defineEmits<{
  favorite: [port: PortInfo];
  copy: [port: PortInfo];
  kill: [port: PortInfo, mode: KillMode];
}>();

const favoriteStore = useFavoriteStore();
</script>
