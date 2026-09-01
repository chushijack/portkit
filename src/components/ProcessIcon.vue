<!--
文件名称：ProcessIcon.vue

文件功能：
按进程名渲染品牌图标。

主要职责：
- 已知开发进程使用 Devicon
- 其余进程回退 Font Awesome

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <span class="wrap" :style="wrapStyle">
    <img
      v-if="isDevicon"
      class="pk-process-icon"
      :src="deviconSrc"
      alt=""
      draggable="false"
    />
    <FaIcon v-else :icon="faIcon" :size="size" />
  </span>
</template>

<script setup lang="ts">
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { faMicrochip } from "@fortawesome/free-solid-svg-icons";
import { computed } from "vue";
import FaIcon from "@/components/FaIcon.vue";
import { processIconOf } from "@/utils/processIcon";

const props = withDefaults(
  defineProps<{
    name: string;
    size?: number;
  }>(),
  { size: 16 },
);

const style = computed(() => processIconOf(props.name));
const isDevicon = computed(() => style.value.kind === "devicon");
const deviconSrc = computed(() => (style.value.kind === "devicon" ? style.value.src : ""));
const faIcon = computed<IconDefinition>(() =>
  style.value.kind === "fa" ? style.value.icon : faMicrochip,
);
const wrapStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}));
</script>

<style scoped>
.wrap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.pk-process-icon {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
</style>
