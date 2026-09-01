<!--
文件名称：ProcessDetail.vue

文件功能：
进程详情页，对应原型图右上详情区域。

主要职责：
- 展示进程元数据、识别服务与资源占用
- 打开目录、复制 PID、优雅关闭与强制结束
- 过长启动命令默认缩略，可展开或复制

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page detail-page">
    <button class="back" type="button" @click="router.back()">
      <FaIcon :icon="faChevronLeft" :size="16" />
      <span>{{ $t("process.title") }}</span>
    </button>

    <a-spin :spinning="processStore.loading" class="detail-spin">
      <div class="detail-body">
        <section v-if="processStore.detail" class="hero pk-card">
        <div class="identity">
          <div
            class="avatar"
            :style="{ color: processIcon.color, background: processIcon.soft }"
          >
            <ProcessIcon :name="processStore.detail.name" :size="22" />
          </div>
          <div class="identity-text">
            <div class="name-row">
              <h2>{{ processStore.detail.name }}</h2>
              <span class="status">
                <span class="status-dot" />
                {{ statusLabel }}
              </span>
            </div>
            <div v-if="serviceLabels.length" class="service-row">
              <span v-for="label in serviceLabels" :key="label" class="service-chip">{{ label }}</span>
            </div>
            <div class="pk-muted subtitle">{{ processStore.detail.displayName }}</div>
          </div>
        </div>
        <div class="hero-actions">
          <a-button class="hero-btn" @click="openDir">
            <template #icon>
              <FaIcon :icon="faFolderOpen" :size="15" />
            </template>
            <span>{{ $t("process.openDir") }}</span>
          </a-button>
          <a-button class="hero-btn" @click="copyPid">
            <template #icon>
              <FaIcon :icon="faCopy" :size="15" />
            </template>
            <span>{{ $t("process.copyPid") }}</span>
          </a-button>
          <a-button class="hero-btn" @click="openKill('graceful')">
            <template #icon>
              <FaIcon :icon="faHand" :size="15" />
            </template>
            <span>{{ $t("process.killGraceful") }}</span>
          </a-button>
          <a-button class="hero-btn hero-btn-kill" danger type="primary" @click="openKill('force')">
            <template #icon>
              <FaIcon :icon="faBolt" :size="15" />
            </template>
            <span>{{ $t("process.killForce") }}</span>
          </a-button>
        </div>
      </section>

      <section v-if="processStore.detail" class="info pk-card">
        <div
          v-for="item in infoItems"
          :key="item.label"
          class="info-item"
          :class="{ wide: item.wide }"
        >
          <div class="info-icon">
            <FaIcon :icon="item.icon" :size="14" />
          </div>
          <div class="info-body">
            <div class="info-label">{{ item.label }}</div>
            <div v-if="item.variant === 'path'" class="file-block">
              <div class="file-name">{{ item.fileName }}</div>
              <div class="code-well">
                <div class="code-text">{{ item.value }}</div>
                <div v-if="item.value !== '-'" class="code-actions">
                  <a-tooltip :title="$t('process.copyPath')">
                    <a-button type="text" class="file-copy" @click="copyPath">
                      <FaIcon :icon="faCopy" :size="13" />
                    </a-button>
                  </a-tooltip>
                </div>
              </div>
            </div>
            <div v-else-if="item.variant === 'command'" class="code-well stack">
              <div
                class="code-text"
                :class="{ collapsed: commandLong && !commandExpanded }"
              >{{ item.value }}</div>
              <div v-if="item.value !== '-'" class="code-actions">
                <button
                  v-if="commandLong"
                  class="command-toggle"
                  type="button"
                  @click="commandExpanded = !commandExpanded"
                >
                  {{ commandExpanded ? $t("process.collapseCommand") : $t("process.expandCommand") }}
                </button>
                <a-tooltip :title="$t('process.copyCommand')">
                  <a-button type="text" class="file-copy" @click="copyCommand">
                    <FaIcon :icon="faCopy" :size="13" />
                  </a-button>
                </a-tooltip>
              </div>
            </div>
            <div v-else class="info-value">{{ item.value }}</div>
          </div>
        </div>
      </section>

      <section v-if="processStore.detail" class="metrics">
        <article class="metric pk-card">
          <div class="metric-head">
            <span class="metric-icon cpu">
              <FaIcon :icon="faMicrochip" :size="14" />
            </span>
            <span>{{ $t("process.cpu") }}</span>
          </div>
          <div class="metric-value">
            {{ processStore.detail.cpuUsage.toFixed(1) }}
            <small>%</small>
          </div>
          <a-progress
            :percent="Math.min(processStore.detail.cpuUsage, 100)"
            :show-info="false"
            stroke-color="#3d7eff"
          />
        </article>
        <article class="metric pk-card">
          <div class="metric-head">
            <span class="metric-icon mem">
              <FaIcon :icon="faMemory" :size="14" />
            </span>
            <span>{{ $t("process.memory") }}</span>
          </div>
          <div class="metric-value">
            {{ formatMemory(processStore.detail.memoryBytes) }}
          </div>
          <a-progress :percent="memoryPercent" :show-info="false" stroke-color="#8b5cf6" />
        </article>
      </section>
      </div>
    </a-spin>

    <KillConfirmModal
      v-model:open="killOpen"
      :loading="killing"
      :mode="pendingMode"
      :port="portFromQuery"
      :process-name="processStore.detail?.name"
      :pid="processStore.detail?.pid"
      @confirm="confirmKill"
    />
  </div>
</template>

<script setup lang="ts">
import {
  faBolt,
  faChevronLeft,
  faClock,
  faCopy,
  faFileLines,
  faFlag,
  faFolderOpen,
  faHand,
  faHashtag,
  faHourglassHalf,
  faMemory,
  faMicrochip,
  faTag,
  faTerminal,
  faUser,
} from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { writeClipboard } from "@/utils/clipboard";
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import FaIcon from "@/components/FaIcon.vue";
import KillConfirmModal from "@/components/KillConfirmModal.vue";
import ProcessIcon from "@/components/ProcessIcon.vue";
import { scanPorts } from "@/composables/useAutoRefresh";
import { usePortStore } from "@/stores/port";
import { useProcessStore } from "@/stores/process";
import type { KillMode, KillOutcome } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { formatMemory } from "@/utils/format";
import { notifyKillOutcome } from "@/utils/killFeedback";
import { displayText, processIconOf } from "@/utils/processIcon";

const props = defineProps<{ pid: string }>();
const processStore = useProcessStore();
const portStore = usePortStore();
const router = useRouter();
const route = useRoute();
const { t } = useI18n();
const killOpen = ref(false);
const killing = ref(false);
const pendingMode = ref<KillMode>("force");
const commandExpanded = ref(false);
const COMMAND_COLLAPSE_CHARS = 180;

const portFromQuery = computed(() => {
  const value = route.query.port;
  return typeof value === "string" ? Number(value) : null;
});

const processIcon = computed(() => processIconOf(processStore.detail?.name ?? ""));

const serviceLabels = computed(() => processStore.detail?.serviceLabels ?? []);

const statusLabel = computed(() => {
  const status = processStore.detail?.status ?? "";
  if (status === "running" || status === "运行中") {
    return t("process.status.running");
  }
  if (status === "exited" || status === "已退出") {
    return t("process.status.exited");
  }
  return status;
});

function priorityLabel(priority: string): string {
  if (priority === "idle" || priority === "低") {
    return t("process.priority.idle");
  }
  if (priority === "belowNormal" || priority === "低于普通") {
    return t("process.priority.belowNormal");
  }
  if (priority === "aboveNormal" || priority === "高于普通") {
    return t("process.priority.aboveNormal");
  }
  if (priority === "high" || priority === "高") {
    return t("process.priority.high");
  }
  if (priority === "realtime" || priority === "实时") {
    return t("process.priority.realtime");
  }
  if (priority === "normal" || priority === "普通") {
    return t("process.priority.normal");
  }
  return displayText(priority);
}

const infoItems = computed(() => {
  const detail = processStore.detail;
  if (!detail) {
    return [];
  }
  return [
    { label: t("process.fields.pid"), value: displayText(detail.pid), icon: faHashtag, wide: false },
    { label: t("process.fields.user"), value: displayText(detail.user), icon: faUser, wide: false },
    { label: t("process.fields.startTime"), value: displayText(detail.startTime), icon: faClock, wide: false },
    { label: t("process.fields.duration"), value: displayText(detail.duration), icon: faHourglassHalf, wide: false },
    { label: t("process.fields.priority"), value: priorityLabel(detail.priority), icon: faFlag, wide: false },
    {
      label: t("process.fields.service"),
      value: serviceLabels.value.length ? serviceLabels.value.join(" · ") : "-",
      icon: faTag,
      wide: false,
    },
    {
      label: t("process.fields.path"),
      value: displayText(detail.path),
      fileName: fileNameOf(detail.path),
      icon: faFileLines,
      wide: true,
      variant: "path" as const,
    },
    {
      label: t("process.fields.command"),
      value: displayText(detail.command),
      icon: faTerminal,
      wide: true,
      variant: "command" as const,
    },
  ];
});

const commandLong = computed(() => {
  const command = processStore.detail?.command?.trim() ?? "";
  return command.length > COMMAND_COLLAPSE_CHARS && command !== "-";
});

const memoryPercent = computed(() => {
  if (!processStore.detail) {
    return 0;
  }
  return Math.min((processStore.detail.memoryBytes / (1024 * 1024 * 1024)) * 10, 100);
});

async function reload(): Promise<void> {
  try {
    await processStore.load(Number(props.pid));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

async function openDir(): Promise<void> {
  try {
    await processStore.openDirectory(Number(props.pid));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

function fileNameOf(path: string): string {
  const trimmed = path.trim();
  if (!trimmed || trimmed === "-") {
    return trimmed || "-";
  }
  const normalized = trimmed.replace(/\\/g, "/");
  const segments = normalized.split("/").filter((part: string) => part.length > 0);
  return segments[segments.length - 1] ?? trimmed;
}

async function copyPath(): Promise<void> {
  const path = processStore.detail?.path?.trim() ?? "";
  if (!path || path === "-") {
    return;
  }
  await writeClipboard(path);
  message.success(t("process.pathCopied"));
}

async function copyCommand(): Promise<void> {
  const command = processStore.detail?.command?.trim() ?? "";
  if (!command || command === "-") {
    return;
  }
  await writeClipboard(command);
  message.success(t("process.commandCopied"));
}

async function copyPid(): Promise<void> {
  await writeClipboard(String(props.pid));
  message.success(t("port.copiedPid"));
}

function openKill(mode: KillMode): void {
  pendingMode.value = mode;
  killOpen.value = true;
}

async function confirmKill(): Promise<void> {
  if (!processStore.detail) {
    return;
  }
  const pid = processStore.detail.pid;
  const port = portFromQuery.value;
  killing.value = true;
  let killError: unknown;
  let killOutcome: KillOutcome | null = null;
  try {
    killOutcome = await processStore.kill({
      pid,
      port,
      protocol: typeof route.query.protocol === "string" ? route.query.protocol : null,
      processName: processStore.detail.name,
      mode: pendingMode.value,
    });
  } catch (error) {
    killError = error;
  }
  try {
    killOpen.value = false;
    await scanPorts({ recordHistory: false });
    if (port !== null) {
      const result = notifyKillOutcome(
        portStore.ports,
        port,
        pid,
        processStore.detail.name,
        {
          still: () =>
            t(pendingMode.value === "graceful" ? "kill.stillGraceful" : "kill.still"),
          watchdog: () => t("errors.watchdog"),
          success: () => t("kill.success"),
        },
        killError,
        killOutcome,
      );
      if (result !== "still") {
        router.push("/");
      }
      return;
    }
    if (killError) {
      message.error(toErrorMessage(killError));
      return;
    }
    message.success(t("kill.success"));
    router.push("/");
  } catch (error) {
    message.error(toErrorMessage(killError ?? error));
  } finally {
    killing.value = false;
  }
}

onMounted(() => {
  void reload();
});

watch(
  () => props.pid,
  () => {
    commandExpanded.value = false;
    void reload();
  },
);

watch(
  () => processStore.detail?.command,
  () => {
    commandExpanded.value = false;
  },
);
</script>

<style scoped>
.detail-page {
  overflow: auto;
  padding: 0 20px 12px 0;
}

.detail-spin {
  flex: 1;
  min-height: 0;
}

.detail-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.back {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  border: 0;
  background: transparent;
  color: var(--pk-text);
  font-size: 20px;
  font-weight: 700;
  padding: 4px 0 8px;
  cursor: pointer;
  width: fit-content;
}

.back:hover {
  color: var(--pk-primary);
}

.hero,
.info {
  padding: 24px;
}

.hero {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.identity {
  display: flex;
  gap: 16px;
  align-items: center;
  min-width: 0;
}

.avatar {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.identity-text {
  min-width: 0;
}

.name-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.name-row h2 {
  margin: 0;
  font-size: 22px;
  line-height: 1.2;
}

.subtitle {
  margin-top: 6px;
}

.service-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.service-chip {
  display: inline-flex;
  align-items: center;
  height: 22px;
  padding: 0 8px;
  border-radius: 6px;
  background: var(--pk-primary-soft);
  color: var(--pk-primary);
  font-size: 12px;
  font-weight: 600;
}

.status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  background: var(--pk-success-soft);
  color: var(--pk-monitor-on);
  font-size: 12px;
  font-weight: 600;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.hero-actions :deep(.hero-btn.ant-btn) {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 14px;
  border-radius: 10px;
  font-weight: 600;
  box-shadow: none;
}

.hero-actions :deep(.hero-btn.ant-btn .ant-btn-icon) {
  display: inline-flex;
  align-items: center;
  margin-inline-end: 0;
}

.hero-actions :deep(.hero-btn.ant-btn-default) {
  color: var(--pk-text);
  background: color-mix(in srgb, var(--pk-text) 4%, var(--pk-card));
  border-color: var(--pk-border);
}

.hero-actions :deep(.hero-btn.ant-btn-default:hover) {
  color: var(--pk-primary);
  border-color: color-mix(in srgb, var(--pk-primary) 45%, var(--pk-border));
  background: var(--pk-primary-soft);
}

.hero-actions :deep(.hero-btn-kill.ant-btn-primary) {
  background: var(--pk-danger);
  border-color: var(--pk-danger);
}

.hero-actions :deep(.hero-btn-kill.ant-btn-primary:hover) {
  filter: brightness(1.06);
}

.info {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.info-item {
  display: flex;
  gap: 14px;
  align-items: flex-start;
  min-width: 0;
  padding: 16px 18px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--pk-text) 4%, var(--pk-card));
  border: 1px solid var(--pk-border);
}

.info-item.wide {
  grid-column: 1 / -1;
}

.info-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--pk-primary-soft);
  color: var(--pk-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.info-body {
  min-width: 0;
}

.info-label {
  font-size: 12px;
  color: var(--pk-text-secondary);
  margin-bottom: 8px;
}

.info-value {
  font-size: 14px;
  font-weight: 600;
  word-break: break-all;
  line-height: 1.5;
}

.file-name {
  font-size: 14px;
  font-weight: 700;
  line-height: 1.4;
  margin-bottom: 8px;
}

.code-well {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--pk-text) 5%, var(--pk-card));
  border: 1px solid var(--pk-border);
}

.code-well.stack {
  flex-direction: column;
}

.code-text {
  flex: 1;
  min-width: 0;
  font-family: "Cascadia Code", Consolas, "SF Mono", ui-monospace, monospace;
  font-size: 12px;
  font-weight: 500;
  line-height: 1.55;
  color: var(--pk-text);
  word-break: break-all;
}

.code-text.collapsed {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  overflow: hidden;
}

.code-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.code-well.stack .code-actions {
  width: 100%;
}

.command-toggle {
  margin-right: auto;
  border: none;
  background: none;
  padding: 0;
  color: var(--pk-primary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
}

.command-toggle:hover {
  text-decoration: underline;
}

.info :deep(.file-copy.ant-btn) {
  width: 28px;
  min-width: 28px;
  height: 28px;
  padding: 0;
  border-radius: 8px;
  color: var(--pk-text-secondary);
  background: transparent;
  border: 1px solid transparent;
  box-shadow: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.info :deep(.file-copy.ant-btn .ant-btn-icon) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-inline-end: 0;
}

.info :deep(.file-copy.ant-btn:hover),
.info :deep(.file-copy.ant-btn:focus-visible) {
  color: var(--pk-primary);
  background: color-mix(in srgb, var(--pk-text) 6%, var(--pk-card));
  border-color: var(--pk-border);
}

.metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.metric {
  padding: 24px;
}

.metric-head {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--pk-text-secondary);
  font-size: 13px;
  font-weight: 500;
}

.metric-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.metric-icon.cpu {
  background: var(--pk-stat-blue-soft);
  color: var(--pk-primary);
}

.metric-icon.mem {
  background: var(--pk-stat-purple-soft);
  color: #8b5cf6;
}

.metric-value {
  margin: 14px 0 12px;
  font-size: 32px;
  font-weight: 700;
  letter-spacing: -0.04em;
  line-height: 1;
}

.metric-value small {
  margin-left: 2px;
  font-size: 16px;
  font-weight: 600;
  color: var(--pk-text-secondary);
}

.metric :deep(.ant-progress-inner) {
  background: color-mix(in srgb, var(--pk-text) 8%, var(--pk-card));
}

.metric :deep(.ant-progress-bg) {
  height: 8px !important;
}
</style>
