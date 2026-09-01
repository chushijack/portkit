<!--
文件名称：FaIcon.vue

文件功能：
统一封装 Font Awesome 图标尺寸与对齐。

主要职责：
- 以 SVG 方式渲染图标
- 让各页面用同一套图标尺寸约定

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <FontAwesomeIcon class="pk-fa" :icon="icon" :style="sizeStyle" />
</template>

<script setup lang="ts">
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    icon: IconDefinition;
    size?: number;
  }>(),
  { size: 16 },
);

const sizeStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}));
</script>

<style scoped>
.pk-fa {
  display: inline-block;
  vertical-align: -0.125em;
}
</style>
