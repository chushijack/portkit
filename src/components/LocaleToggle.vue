<!--
文件名称：LocaleToggle.vue

文件功能：
顶栏界面语言切换按钮。

主要职责：
- 在顶栏提供与设置页相同的语言选项
- 选中后立即切换并保存语言

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-tooltip :title="$t('header.language')">
    <span class="toggle-wrap">
      <a-dropdown
        placement="bottomRight"
        :trigger="['click']"
        overlay-class-name="pk-port-actions-dropdown"
        :get-popup-container="popupToBody"
      >
        <a-button type="text" class="toggle">
          <FaIcon :icon="faLanguage" :size="16" />
        </a-button>
        <template #overlay>
          <a-menu :selected-keys="[settingsStore.settings.locale]" @click="onSelect">
            <a-menu-item v-for="item in localeOptions" :key="item.value">
              {{ $t(item.labelKey) }}
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </span>
  </a-tooltip>
</template>

<script setup lang="ts">
import { faLanguage } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import FaIcon from "@/components/FaIcon.vue";
import { useSettingsStore } from "@/stores/settings";
import { isLanguageMode, type LanguageMode } from "@/types";
import { toErrorMessage } from "@/utils/error";

const localeOptions: { value: LanguageMode; labelKey: "settings.languageSystem" | "settings.languageZh" | "settings.languageEn" | "settings.languageJa" }[] = [
  { value: "system", labelKey: "settings.languageSystem" },
  { value: "zh-CN", labelKey: "settings.languageZh" },
  { value: "en", labelKey: "settings.languageEn" },
  { value: "ja", labelKey: "settings.languageJa" },
];

const settingsStore = useSettingsStore();

function popupToBody(): HTMLElement {
  return document.body;
}

async function onSelect(info: { key: string | number }): Promise<void> {
  const next = String(info.key);
  if (!isLanguageMode(next) || next === settingsStore.settings.locale) {
    return;
  }
  try {
    await settingsStore.setLocale(next);
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}
</script>

<style scoped>
.toggle-wrap,
.toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
</style>
