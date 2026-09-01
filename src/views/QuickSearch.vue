<!--
文件名称：QuickSearch.vue

文件功能：
全局快捷键唤起的快速搜索窗口。

主要职责：
- 按端口、PID、进程名即时过滤
- 一键清空搜索词
- Esc 在窗口级关闭；失焦后由后端自动隐藏
- 查看详情、复制 PID、结束进程

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-config-provider :theme="antdTheme" :locale="antdLocale">
    <div class="quick" data-tauri-drag-region>
    <a-input
      ref="inputRef"
      v-model:value="keyword"
      size="large"
      allow-clear
      :placeholder="$t('quick.placeholder')"
      @keydown.enter="onEnter"
    >
      <template #prefix>
        <FaIcon :icon="faMagnifyingGlass" :size="16" />
      </template>
    </a-input>

    <div class="list" data-tauri-drag-region="false" @mousedown="focusSearch">
      <button
        v-for="(item, index) in results"
        :key="`${item.protocol}-${item.port}-${item.pid}`"
        class="item"
        :class="{ active: index === activeIndex }"
        @mouseenter="activeIndex = index"
        @click="openDetail(item)"
      >
        <div class="item-main">
          <ProcessIcon :name="item.processName" :size="18" />
          <div>
            <b>{{ item.port }}</b>
            <span class="label">{{ item.serviceLabel ?? item.processName }}</span>
          </div>
        </div>
        <div class="meta">
          {{ item.protocol }} · PID {{ item.pid }} · {{ item.processName }} · {{ item.status }}
        </div>
      </button>
      <a-empty v-if="results.length === 0" :description="$t('quick.empty')" />
    </div>

    <footer class="hints">
      <span>{{ $t("quick.enterDetail") }}</span>
      <span>{{ $t("quick.copyPid", { mod: modifierLabel(isMac) }) }}</span>
      <span>{{ $t("quick.killGraceful", { mod: modifierLabel(isMac) }) }}</span>
      <span>{{ $t("quick.killForce", { mod: modifierLabel(isMac) }) }}</span>
      <span>{{ $t("quick.close") }}</span>
    </footer>

    <KillConfirmModal
      v-model:open="killOpen"
      :loading="killing"
      :mode="pendingMode"
      :port="pending?.port"
      :process-name="pending?.processName"
      :pid="pending?.pid"
      @confirm="confirmKill"
    />
    </div>
  </a-config-provider>
</template>

<script setup lang="ts">
import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { writeClipboard } from "@/utils/clipboard";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import FaIcon from "@/components/FaIcon.vue";
import KillConfirmModal from "@/components/KillConfirmModal.vue";
import ProcessIcon from "@/components/ProcessIcon.vue";
import { windowApi } from "@/api";
import { useAntdLocale } from "@/composables/useAntdLocale";
import { useAntdTheme } from "@/composables/useAntdTheme";
import { usePortStore } from "@/stores/port";
import { useProcessStore } from "@/stores/process";
import { useSettingsStore } from "@/stores/settings";
import type { KillMode, KillOutcome, PortInfo } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { modifierLabel } from "@/utils/format";
import { notifyKillOutcome } from "@/utils/killFeedback";
import { isMacPlatform } from "@/utils/platform";
import { getCurrentWebviewWindowOrNull, isTauriRuntime } from "@/utils/tauri";

const portStore = usePortStore();
const processStore = useProcessStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const antdTheme = useAntdTheme();
const antdLocale = useAntdLocale();
const keyword = ref("");
const activeIndex = ref(0);
const inputRef = ref<{ focus: () => void } | null>(null);
const isMac = isMacPlatform();
const killOpen = ref(false);
const killing = ref(false);
const pending = ref<PortInfo | null>(null);
const pendingMode = ref<KillMode>("force");
const quickWindow = getCurrentWebviewWindowOrNull();
let stopFocusListener: (() => void) | null = null;

const results = computed(() => {
  const query = keyword.value.trim().toLowerCase();
  if (!query) {
    return portStore.ports.slice(0, 8);
  }
  return portStore.ports
    .filter((item) => {
      return (
        String(item.port).includes(query) ||
        String(item.pid).includes(query) ||
        item.processName.toLowerCase().includes(query) ||
        (item.serviceLabel ?? "").toLowerCase().includes(query)
      );
    })
    .slice(0, 8);
});

watch(keyword, () => {
  activeIndex.value = 0;
});

async function close(): Promise<void> {
  await windowApi.hideQuickSearch();
}

function focusSearch(): void {
  inputRef.value?.focus();
}

async function openDetail(item: PortInfo): Promise<void> {
  await windowApi.openMainRoute(`/process/${item.pid}?port=${item.port}&protocol=${item.protocol}`);
}

function onEnter(event: KeyboardEvent): void {
  const current = results.value[activeIndex.value];
  if (!current) {
    return;
  }
  if (event.metaKey || event.ctrlKey) {
    event.preventDefault();
    pending.value = current;
    pendingMode.value = event.shiftKey ? "force" : "graceful";
    killOpen.value = true;
    return;
  }
  void openDetail(current);
}

async function confirmKill(): Promise<void> {
  if (!pending.value) {
    return;
  }
  killing.value = true;
  const target = pending.value;
  let killError: unknown;
  let killOutcome: KillOutcome | null = null;
  try {
    killOutcome = await processStore.kill({
      pid: target.pid,
      port: target.port,
      protocol: target.protocol,
      processName: target.processName,
      mode: pendingMode.value,
    });
  } catch (error) {
    killError = error;
  }
  try {
    killOpen.value = false;
    await portStore.scan({ recordHistory: false });
    notifyKillOutcome(
      portStore.ports,
      target.port,
      target.pid,
      target.processName,
      {
        still: () =>
          t(pendingMode.value === "graceful" ? "kill.stillGraceful" : "kill.still"),
        watchdog: () => t("errors.watchdog"),
        success: () => t("kill.success"),
      },
      killError,
      killOutcome,
    );
  } catch (error) {
    message.error(toErrorMessage(killError ?? error));
  } finally {
    killing.value = false;
  }
}

async function onKeydown(event: KeyboardEvent): Promise<void> {
  if (event.key === "Escape") {
    if (killOpen.value) {
      return;
    }
    event.preventDefault();
    await close();
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    activeIndex.value = Math.min(activeIndex.value + 1, Math.max(results.value.length - 1, 0));
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    activeIndex.value = Math.max(activeIndex.value - 1, 0);
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "c") {
    const current = results.value[activeIndex.value];
    if (current) {
      await writeClipboard(String(current.pid));
      message.success(t("port.copiedPid"));
    }
  }
}

onMounted(async () => {
  await settingsStore.load();
  if (!isTauriRuntime()) {
    focusSearch();
    window.addEventListener("keydown", onKeydown);
    return;
  }

  await portStore.scan({ recordHistory: false });
  focusSearch();
  window.addEventListener("keydown", onKeydown);
  if (quickWindow) {
    stopFocusListener = await quickWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        window.setTimeout(focusSearch, 0);
        return;
      }
      keyword.value = "";
      activeIndex.value = 0;
    });
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  stopFocusListener?.();
});
</script>

<style scoped>
.quick {
  width: 100%;
  height: 100%;
  background: var(--pk-card);
  display: flex;
  flex-direction: column;
  padding: 16px;
}

.list {
  flex: 1;
  overflow: auto;
  margin: 12px 0;
}

.item {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
}

.item-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.item.active {
  background: var(--pk-primary-soft);
}

.label {
  margin-left: 8px;
  color: var(--pk-text-secondary);
}

.meta {
  margin-top: 4px;
  font-size: 12px;
  color: var(--pk-text-secondary);
}

.hints {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  font-size: 12px;
  color: var(--pk-text-secondary);
}
</style>
