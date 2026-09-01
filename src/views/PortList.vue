<!--
文件名称：PortList.vue

文件功能：
端口列表主页面，对应原型图主界面。

主要职责：
- 展示统计卡片与端口表格
- 提供扫描、监控和常用端口操作
- 底部数量跟随搜索结果

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page">
    <StatCards />
    <div class="table-wrap pk-card">
      <PortTable
        @detail="goDetail"
        @favorite="onFavorite"
        @kill="openKill"
        @copy="copyPid"
      />
    </div>
    <footer class="footer">
      <a-button type="primary" size="large" :loading="portStore.loading" @click="manualScan">
        {{ $t("port.scan") }}
      </a-button>
      <a-button
        size="large"
        :type="monitorStore.running ? 'primary' : 'default'"
        :ghost="monitorStore.running"
        class="monitor-btn"
        @click="toggleMonitor"
      >
        {{ monitorStore.running ? $t("port.monitorOn") : $t("port.monitorOff") }}
      </a-button>
      <a-dropdown v-model:open="commonMenuOpen" overlay-class-name="pk-common-ports-dropdown">
        <a-button size="large" class="common-btn">
          <span>{{ $t("port.commonPorts") }}</span>
          <FaIcon
            class="caret"
            :class="{ open: commonMenuOpen }"
            :icon="faAngleDown"
            :size="14"
          />
        </a-button>
        <template #overlay>
          <a-menu @click="onCommon">
            <a-menu-item key="common">{{ $t("port.scanCommon") }}</a-menu-item>
            <a-menu-item v-for="item in portStore.commonPorts" :key="String(item.port)">
              <span class="common-port-row">
                <span class="common-port">{{ item.port }}</span>
                <span class="common-note">{{ item.label }}</span>
              </span>
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
      <span class="count">{{ footerCount }}</span>
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
</template>

<script setup lang="ts">
import { faAngleDown } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { writeClipboard } from "@/utils/clipboard";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import FaIcon from "@/components/FaIcon.vue";
import KillConfirmModal from "@/components/KillConfirmModal.vue";
import PortTable from "@/components/PortTable.vue";
import StatCards from "@/components/StatCards.vue";
import { scanPorts } from "@/composables/useAutoRefresh";
import { useFavoriteStore } from "@/stores/favorite";
import { useMonitorStore } from "@/stores/monitor";
import { usePortStore } from "@/stores/port";
import { useProcessStore } from "@/stores/process";
import type { KillMode, KillOutcome, PortInfo } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { notifyKillOutcome } from "@/utils/killFeedback";

const portStore = usePortStore();
const monitorStore = useMonitorStore();
const favoriteStore = useFavoriteStore();
const processStore = useProcessStore();
const router = useRouter();
const { t } = useI18n();

const killOpen = ref(false);
const killing = ref(false);
const commonMenuOpen = ref(false);
const pending = ref<PortInfo | null>(null);
const pendingMode = ref<KillMode>("force");

const footerCount = computed(() => {
  const total = portStore.stats.total;
  const filtered = portStore.filteredPorts.length;
  if (portStore.keyword.trim()) {
    return t("port.filteredCount", { filtered, total });
  }
  return t("port.totalCount", { total });
});

async function manualScan(): Promise<void> {
  try {
    await scanPorts({ recordHistory: true });
    message.success(t("port.scanDone"));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

function toggleMonitor(): void {
  if (monitorStore.running) {
    monitorStore.stop();
    return;
  }
  monitorStore.start();
  void scanPorts({ recordHistory: false });
}

function goDetail(port: PortInfo): void {
  void router.push({
    name: "process",
    params: { pid: String(port.pid) },
    query: { port: String(port.port), protocol: port.protocol },
  });
}

async function onFavorite(port: PortInfo): Promise<void> {
  try {
    if (favoriteStore.isFavorite(port.port)) {
      await favoriteStore.remove(port.port);
      message.success(t("port.unfavorited"));
      return;
    }
    const note = port.serviceLabel ?? port.processName;
    await favoriteStore.add(port.port, port.protocol, note);
    message.success(t("port.favorited"));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

function openKill(port: PortInfo, mode: KillMode): void {
  pending.value = port;
  pendingMode.value = mode;
  killOpen.value = true;
}

async function confirmKill(): Promise<void> {
  if (!pending.value) {
    message.warning(t("port.noProcess"));
    return;
  }
  const target = pending.value;
  killing.value = true;
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
    await scanPorts({ recordHistory: false });
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

async function copyPid(port: PortInfo): Promise<void> {
  await writeClipboard(String(port.pid));
  message.success(t("port.copiedPid"));
}

async function onCommon(info: { key: string | number }): Promise<void> {
  const key = String(info.key);
  try {
    if (key === "common") {
      await scanPorts({ mode: "common", recordHistory: true });
    } else {
      const port = Number(key);
      await scanPorts({
        mode: "custom",
        startPort: port,
        endPort: port,
        recordHistory: true,
      });
    }
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}
</script>

<style scoped>
.table-wrap {
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 4px 16px 8px 8px;
}

.footer {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: auto;
}

.monitor-btn {
  color: #16a34a;
  border-color: #16a34a;
}

.footer :deep(.common-btn) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.caret {
  color: var(--pk-text-secondary);
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

.caret.open {
  transform: rotate(180deg);
}

.count {
  margin-left: auto;
  color: var(--pk-text-secondary);
}
</style>
