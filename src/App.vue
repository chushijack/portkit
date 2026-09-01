<!--
文件名称：App.vue

文件功能：
主窗口根组件，负责主题、布局与全局事件。

主要职责：
- 应用 Ant Design 主题与语言
- 启动时加载设置并扫描
- 按设置自动检查更新
- 响应托盘与路由跳转事件

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-config-provider :theme="antdTheme" :locale="antdLocale">
    <AppLayout @refresh="onRefresh" />
    <UpdateDialog />
  </a-config-provider>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import AppLayout from "@/components/AppLayout.vue";
import UpdateDialog from "@/components/UpdateDialog.vue";
import { useAntdLocale } from "@/composables/useAntdLocale";
import { useAntdTheme } from "@/composables/useAntdTheme";
import { useAutoRefresh } from "@/composables/useAutoRefresh";
import { useFavoriteStore } from "@/stores/favorite";
import { usePortStore } from "@/stores/port";
import { useSettingsStore } from "@/stores/settings";
import { useUpdateStore } from "@/stores/update";
import { isTauriRuntime, listenSafe } from "@/utils/tauri";
import type { UnlistenFn } from "@tauri-apps/api/event";

const settingsStore = useSettingsStore();
const favoriteStore = useFavoriteStore();
const portStore = usePortStore();
const updateStore = useUpdateStore();
const { scan } = useAutoRefresh();
const router = useRouter();
const antdTheme = useAntdTheme();
const antdLocale = useAntdLocale();
const unlistens: UnlistenFn[] = [];

async function onRefresh(): Promise<void> {
  await scan({ recordHistory: true });
}

onMounted(async () => {
  await settingsStore.load();
  if (!isTauriRuntime()) {
    return;
  }

  await favoriteStore.load();
  await portStore.loadCommonPorts();
  await updateStore.loadVersion();
  if (settingsStore.settings.notifications) {
    try {
      const { isPermissionGranted, requestPermission } = await import(
        "@tauri-apps/plugin-notification"
      );
      const granted = await isPermissionGranted();
      if (!granted) {
        await requestPermission();
      }
    } catch {
      // 通知权限失败不影响主流程
    }
  }
  if (settingsStore.settings.autoScanOnStartup) {
    await scan({ recordHistory: true });
  }
  if (settingsStore.settings.autoCheckUpdate) {
    try {
      const result = await updateStore.check();
      if (result.available) {
        updateStore.openDialog();
      }
    } catch {
      // 启动时检查失败不打断主流程
    }
  }

  const navigateUnlisten = await listenSafe<string>("navigate", (event) => {
    void router.push(event.payload);
  });
  if (navigateUnlisten) {
    unlistens.push(navigateUnlisten);
  }

  const trayUnlisten = await listenSafe<string>("tray-action", (event) => {
    if (event.payload === "scan") {
      void scan({ recordHistory: true });
    }
  });
  if (trayUnlisten) {
    unlistens.push(trayUnlisten);
  }
});

onUnmounted(() => {
  unlistens.forEach((item) => item());
  updateStore.dispose();
});
</script>
