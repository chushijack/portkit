<!--
文件名称：AppHeader.vue

文件功能：
主界面顶部搜索与刷新工具栏。

主要职责：
- 搜索端口、进程名、PID
- 手动刷新与切换自动刷新间隔
- 切换亮色 / 暗色
- 切换界面语言
- 打开 GitHub 仓库
- 快捷进入设置

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <header class="header">
    <a-input
      v-model:value="portStore.keyword"
      class="search"
      :placeholder="$t('header.searchPlaceholder')"
      allow-clear
    >
      <template #prefix>
        <FaIcon :icon="faMagnifyingGlass" :size="16" />
      </template>
      <template #suffix>
        <span class="shortcut">{{ modifierLabel(isMac) }}+K</span>
      </template>
    </a-input>

    <div class="actions">
      <a-tooltip :title="$t('header.refresh')">
      <a-button type="text" :loading="portStore.loading" @click="emit('refresh')">
          <FaIcon :icon="faRotate" :size="16" />
        </a-button>
      </a-tooltip>
      <a-dropdown>
        <a-button>
          {{ $t("header.autoRefresh", { seconds: settingsStore.settings.refreshInterval }) }}
          <FaIcon :icon="faChevronDown" :size="12" />
        </a-button>
        <template #overlay>
          <a-menu
            :selected-keys="[String(settingsStore.settings.refreshInterval)]"
            @click="onInterval"
          >
            <a-menu-item v-for="item in intervals" :key="String(item)">
              {{ $t("header.seconds", { n: item }) }}
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
      <div class="icon-actions">
      <a-tooltip :title="$t('header.github')">
        <a-button type="text" class="github-btn" @click="openGithub">
          <svg class="github-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="currentColor"
              d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8"
            />
          </svg>
        </a-button>
      </a-tooltip>
      <LocaleToggle />
      <ThemeToggle />
      <a-button type="text" @click="router.push('/settings')">
        <FaIcon :icon="faGear" :size="16" />
      </a-button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { faChevronDown, faGear, faMagnifyingGlass, faRotate } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { useRouter } from "vue-router";
import FaIcon from "@/components/FaIcon.vue";
import LocaleToggle from "@/components/LocaleToggle.vue";
import ThemeToggle from "@/components/ThemeToggle.vue";
import { windowApi } from "@/api";
import { useHotkeys } from "@/composables/useHotkeys";
import { usePortStore } from "@/stores/port";
import { useSettingsStore } from "@/stores/settings";
import { toErrorMessage } from "@/utils/error";
import { modifierLabel } from "@/utils/format";
import { isMacPlatform } from "@/utils/platform";

const emit = defineEmits<{ refresh: [] }>();

const portStore = usePortStore();
const settingsStore = useSettingsStore();
const router = useRouter();
const isMac = isMacPlatform();
const intervals = [1, 3, 5, 10];

useHotkeys(() => {
  const root = document.querySelector(".search input");
  if (root instanceof HTMLInputElement) {
    root.focus();
  }
});

async function onInterval(info: { key: string | number }): Promise<void> {
  await settingsStore.patch({ refreshInterval: Number(info.key) });
}

async function openGithub(): Promise<void> {
  try {
    await windowApi.openGithub();
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}
</script>

<style scoped>
.header {
  height: var(--pk-header-height);
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 20px;
  background: var(--pk-card);
  border-bottom: 1px solid var(--pk-border);
}

.search {
  flex: 1;
  max-width: 560px;
}

.shortcut {
  font-size: 12px;
  color: var(--pk-text-secondary);
  background: var(--pk-chip-bg);
  border-radius: 6px;
  padding: 1px 6px;
}

.actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.actions :deep(.ant-btn-text) {
  width: 32px;
  min-width: 32px;
  padding-inline: 0;
}

.github-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.github-icon {
  width: 16px;
  height: 16px;
  display: block;
}
</style>
