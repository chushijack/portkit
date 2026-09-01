<!--
文件名称：Settings.vue

文件功能：
设置页面，对应原型图底部设置区域。

主要职责：
- 扫描设置
- 系统设置
- 外观设置
- 界面语言
- 软件更新
- 快捷键说明

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page">
    <h1 class="pk-page-title">{{ $t("settings.title") }}</h1>
    <a-tabs v-model:activeKey="activeKey" class="tabs pk-card">
      <a-tab-pane key="scan" :tab="$t('settings.tabs.scan')">
        <a-form layout="vertical" class="form">
          <a-form-item :label="$t('settings.scanRange')">
            <a-radio-group v-model:value="form.scanMode">
              <a-radio value="all">{{ $t("settings.scanAll") }}</a-radio>
              <a-radio value="common">{{ $t("settings.scanCommon") }}</a-radio>
              <a-radio value="custom">{{ $t("settings.scanCustom") }}</a-radio>
            </a-radio-group>
            <div v-if="form.scanMode === 'custom'" class="range">
              <a-input-number v-model:value="form.customStart" :min="1" :max="65535" />
              <span>—</span>
              <a-input-number v-model:value="form.customEnd" :min="1" :max="65535" />
            </div>
          </a-form-item>
          <a-form-item :label="$t('settings.refreshInterval')">
            <a-select v-model:value="form.refreshInterval" style="width: 200px">
              <a-select-option :value="1">{{ $t("settings.intervalOption", { n: 1 }) }}</a-select-option>
              <a-select-option :value="3">{{ $t("settings.intervalOption", { n: 3 }) }}</a-select-option>
              <a-select-option :value="5">{{ $t("settings.intervalOption", { n: 5 }) }}</a-select-option>
              <a-select-option :value="10">{{ $t("settings.intervalOption", { n: 10 }) }}</a-select-option>
            </a-select>
          </a-form-item>
          <a-form-item :label="$t('settings.autoScan')">
            <a-switch v-model:checked="form.autoScanOnStartup" />
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="system" :tab="$t('settings.tabs.system')">
        <a-form layout="vertical" class="form">
          <a-form-item :label="$t('settings.autostart')">
            <a-switch v-model:checked="form.autostart" />
          </a-form-item>
          <a-form-item :label="$t('settings.backgroundRun')">
            <a-switch v-model:checked="form.backgroundRun" />
            <div class="pk-muted">{{ $t("settings.backgroundHint") }}</div>
          </a-form-item>
          <a-form-item :label="$t('settings.notifications')">
            <a-switch v-model:checked="form.notifications" />
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="update" :tab="$t('settings.tabs.update')">
        <a-form layout="vertical" class="form">
          <a-form-item :label="$t('settings.currentVersion')">
            <div class="version">{{ updateStore.currentVersion || "-" }}</div>
          </a-form-item>
          <a-form-item :label="$t('settings.autoCheckUpdate')">
            <a-switch :checked="settingsStore.settings.autoCheckUpdate" @update:checked="onAutoCheckUpdate" />
            <div class="pk-muted">{{ $t("settings.autoCheckUpdateHint") }}</div>
          </a-form-item>
          <a-form-item>
            <a-space>
              <a-button type="primary" :loading="updateStore.status === 'checking'" @click="onCheckUpdate">
                {{ $t("settings.checkUpdate") }}
              </a-button>
              <a-button @click="onOpenChangelog">{{ $t("settings.viewChangelog") }}</a-button>
            </a-space>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="appearance" :tab="$t('settings.tabs.appearance')">
        <a-form layout="vertical" class="form">
          <a-form-item :label="$t('settings.theme')">
            <a-radio-group :value="settingsStore.settings.theme" @update:value="onThemeUpdate">
              <a-radio value="light">{{ $t("settings.themeLight") }}</a-radio>
              <a-radio value="dark">{{ $t("settings.themeDark") }}</a-radio>
              <a-radio value="system">{{ $t("settings.themeSystem") }}</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item :label="$t('settings.language')">
            <a-radio-group :value="settingsStore.settings.locale" @update:value="onLocaleUpdate">
              <a-radio value="system">{{ $t("settings.languageSystem") }}</a-radio>
              <a-radio value="zh-CN">{{ $t("settings.languageZh") }}</a-radio>
              <a-radio value="en">{{ $t("settings.languageEn") }}</a-radio>
              <a-radio value="ja">{{ $t("settings.languageJa") }}</a-radio>
            </a-radio-group>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="shortcuts" :tab="$t('settings.tabs.shortcuts')">
        <a-descriptions bordered :column="1" class="form">
          <a-descriptions-item :label="`${mod} + Shift + P`">{{ $t("settings.shortcutQuickSearch") }}</a-descriptions-item>
          <a-descriptions-item :label="`${mod} + K`">{{ $t("settings.shortcutFocusSearch") }}</a-descriptions-item>
          <a-descriptions-item :label="`${mod} + R`">{{ $t("settings.shortcutTrayScan") }}</a-descriptions-item>
          <a-descriptions-item :label="`${mod} + Q`">{{ $t("settings.shortcutQuit") }}</a-descriptions-item>
        </a-descriptions>
      </a-tab-pane>
    </a-tabs>
    <div v-if="showSave" class="actions">
      <a-button type="primary" :loading="saving" @click="onSave">{{ $t("settings.save") }}</a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { message } from "ant-design-vue";
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { windowApi } from "@/api";
import { useSettingsStore } from "@/stores/settings";
import { useUpdateStore } from "@/stores/update";
import { isLanguageMode, isThemeMode, type AppSettings } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { modifierLabel } from "@/utils/format";
import { isMacPlatform } from "@/utils/platform";

const settingsStore = useSettingsStore();
const updateStore = useUpdateStore();
const { t } = useI18n();
const activeKey = ref("scan");
const saving = ref(false);
const form = reactive<AppSettings>({ ...settingsStore.settings });
const mod = modifierLabel(isMacPlatform());
const showSave = computed(() => activeKey.value === "scan" || activeKey.value === "system");

watch(
  () => settingsStore.settings,
  (value) => {
    Object.assign(form, value);
  },
  { deep: true, immediate: true },
);

onMounted(() => {
  void updateStore.loadVersion();
});

async function onThemeUpdate(value: unknown): Promise<void> {
  if (typeof value !== "string" || !isThemeMode(value) || value === settingsStore.settings.theme) {
    return;
  }
  try {
    await settingsStore.setTheme(value);
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function onLocaleUpdate(value: unknown): Promise<void> {
  if (typeof value !== "string" || !isLanguageMode(value) || value === settingsStore.settings.locale) {
    return;
  }
  try {
    await settingsStore.setLocale(value);
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function onAutoCheckUpdate(checked: boolean): Promise<void> {
  if (checked === settingsStore.settings.autoCheckUpdate) {
    return;
  }
  try {
    await settingsStore.patch({ autoCheckUpdate: checked });
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function onCheckUpdate(): Promise<void> {
  try {
    const result = await updateStore.check();
    if (result.available) {
      updateStore.openDialog();
      return;
    }
    message.success(t("settings.upToDate"));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function onOpenChangelog(): Promise<void> {
  try {
    await windowApi.openChangelog();
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function onSave(): Promise<void> {
  saving.value = true;
  try {
    await settingsStore.save({
      ...form,
      theme: settingsStore.settings.theme,
      locale: settingsStore.settings.locale,
      autoCheckUpdate: settingsStore.settings.autoCheckUpdate,
    });
    message.success(t("settings.saved"));
  } catch (error) {
    message.error(toErrorMessage(error));
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.tabs {
  flex: 1;
  padding: 8px 20px 20px;
  overflow: auto;
}

.form {
  max-width: 640px;
  padding-top: 12px;
}

.range {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.actions {
  display: flex;
  justify-content: flex-end;
}

.version {
  font-variant-numeric: tabular-nums;
}
</style>
