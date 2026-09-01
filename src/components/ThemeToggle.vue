<!--
文件名称：ThemeToggle.vue

文件功能：
顶栏亮色 / 暗色切换按钮。

主要职责：
- 点击后切换主题并播放过渡动画
- 用太阳 / 月亮图标提示当前模式

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-tooltip :title="tooltip">
    <a-button type="text" class="toggle" @click="onToggle">
      <Transition name="theme-icon" mode="out-in">
        <span :key="settingsStore.resolvedTheme" class="icon-wrap">
          <FaIcon
            :icon="settingsStore.resolvedTheme === 'dark' ? faSun : faMoon"
            :size="16"
          />
        </span>
      </Transition>
    </a-button>
  </a-tooltip>
</template>

<script setup lang="ts">
import { faMoon, faSun } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import FaIcon from "@/components/FaIcon.vue";
import { useSettingsStore } from "@/stores/settings";
import { toErrorMessage } from "@/utils/error";

const settingsStore = useSettingsStore();
const { t } = useI18n();

const tooltip = computed(() =>
  settingsStore.resolvedTheme === "dark" ? t("theme.toLight") : t("theme.toDark"),
);

async function onToggle(event: MouseEvent): Promise<void> {
  const next = settingsStore.resolvedTheme === "dark" ? "light" : "dark";
  try {
    await settingsStore.setTheme(next, { x: event.clientX, y: event.clientY });
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}
</script>

<style scoped>
.toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.icon-wrap {
  display: inline-flex;
  width: 18px;
  height: 18px;
}

.theme-icon-enter-active,
.theme-icon-leave-active {
  transition:
    opacity 0.28s ease,
    transform 0.36s cubic-bezier(0.22, 1, 0.36, 1);
}

.theme-icon-enter-from {
  opacity: 0;
  transform: rotate(-70deg) scale(0.6);
}

.theme-icon-leave-to {
  opacity: 0;
  transform: rotate(70deg) scale(0.6);
}
</style>
