<!--
文件名称：PortTable.vue

文件功能：
端口列表表格与行内操作。

主要职责：
- 分页展示端口字段
- 用轻量标签区分协议、状态与识别服务
- 点击整行查看进程详情
- 提供详情、收藏、优雅关闭与强制结束入口
- 右键行打开与更多操作相同的菜单

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-dropdown
    :trigger="['contextmenu']"
    :open="contextOpen"
    overlay-class-name="pk-port-actions-dropdown"
    :get-popup-container="popupToBody"
    :destroy-popup-on-hide="true"
    @openChange="onContextOpenChange"
  >
    <div class="table-context-host" @contextmenu.capture="onHostContextMenu">
      <a-table
        class="table"
        :columns="columns"
        :data-source="portStore.filteredPorts"
        :loading="portStore.loading"
        :pagination="pagination"
        :row-key="rowKey"
        :custom-row="customRow"
        size="middle"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'port'">
            <div class="port-cell">
              <span class="port">{{ record.port }}</span>
              <a-tooltip v-if="record.serviceLabel" :title="record.serviceLabel">
                <span class="service">{{ shortServiceLabel(record.serviceLabel) }}</span>
              </a-tooltip>
            </div>
          </template>
          <template v-else-if="column.key === 'protocol'">
            <span class="pill" :class="record.protocol === 'UDP' ? 'pill-udp' : 'pill-tcp'">
              {{ record.protocol }}
            </span>
          </template>
          <template v-else-if="column.key === 'status'">
            <span class="pill pill-status">
              <span class="pill-dot" />
              {{ record.status }}
            </span>
          </template>
          <template v-else-if="column.key === 'listenAddress'">
            <span class="mono">{{ record.listenAddress }}</span>
          </template>
          <template v-else-if="isProcessName(column)">
            <span class="process-cell">
              <span class="process-icon">
                <ProcessIcon :name="record.processName" :size="16" />
              </span>
              <span class="process-name">{{ record.processName }}</span>
            </span>
          </template>
          <template v-else-if="column.key === 'pid'">
            <span class="num">{{ record.pid }}</span>
          </template>
          <template v-else-if="column.key === 'occupiedTime'">
            <span class="num">{{ record.occupiedTime }}</span>
          </template>
          <template v-else-if="column.key === 'actions'">
            <a-space :size="0" class="actions" @click.stop>
              <a-tooltip :title="$t('port.viewDetail')">
                <a-button type="text" class="action-btn" @click="emit('detail', record)">
                  <FaIcon :icon="faEye" :size="14" />
                </a-button>
              </a-tooltip>
              <a-dropdown
                placement="bottomRight"
                :trigger="['click']"
                overlay-class-name="pk-port-actions-dropdown"
                :get-popup-container="popupToBody"
              >
                <a-button type="text" class="action-btn">
                  <FaIcon :icon="faEllipsis" :size="14" />
                </a-button>
                <template #overlay>
                  <PortRowActionMenu
                    :record="record"
                    @favorite="emit('favorite', $event)"
                    @copy="emit('copy', $event)"
                    @kill="onKill"
                  />
                </template>
              </a-dropdown>
            </a-space>
          </template>
        </template>
      </a-table>
    </div>
    <template #overlay>
      <PortRowActionMenu
        v-if="contextRecord"
        :record="contextRecord"
        @favorite="onContextAction('favorite', $event)"
        @copy="onContextAction('copy', $event)"
        @kill="onKill"
      />
      <a-menu v-else :selectable="false" />
    </template>
  </a-dropdown>
</template>

<script setup lang="ts">
import { faEllipsis, faEye } from "@fortawesome/free-solid-svg-icons";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import FaIcon from "@/components/FaIcon.vue";
import PortRowActionMenu from "@/components/PortRowActionMenu.vue";
import ProcessIcon from "@/components/ProcessIcon.vue";
import type { KillMode, PortInfo } from "@/types";
import { usePortStore } from "@/stores/port";
import { portKey } from "@/utils/format";

type RowAction = "favorite" | "copy";

const emit = defineEmits<{
  detail: [port: PortInfo];
  favorite: [port: PortInfo];
  kill: [port: PortInfo, mode: KillMode];
  copy: [port: PortInfo];
}>();

const portStore = usePortStore();
const { t } = useI18n();
const currentPage = ref(1);
const pageSize = ref(10);
const contextOpen = ref(false);
const contextRecord = ref<PortInfo | null>(null);
const suppressRowClick = ref(false);

const pagination = computed(() => ({
  current: currentPage.value,
  pageSize: pageSize.value,
  showSizeChanger: true,
  showQuickJumper: true,
  pageSizeOptions: ["10", "20", "50", "100"],
  showTotal: (total: number) => t("port.showTotal", { total }),
}));

const columns = computed(() => [
  { title: t("port.columns.port"), key: "port", dataIndex: "port", width: 156 },
  { title: t("port.columns.protocol"), key: "protocol", dataIndex: "protocol", width: 84 },
  { title: t("port.columns.status"), key: "status", dataIndex: "status", width: 108 },
  { title: t("port.columns.listenAddress"), key: "listenAddress", dataIndex: "listenAddress", ellipsis: true, width: 188 },
  { title: t("port.columns.processName"), key: "processName", dataIndex: "processName", ellipsis: true },
  { title: t("port.columns.pid"), key: "pid", dataIndex: "pid", width: 84 },
  { title: t("port.columns.occupiedTime"), key: "occupiedTime", dataIndex: "occupiedTime", width: 112 },
  { title: t("port.columns.actions"), key: "actions", width: 92, align: "right" as const },
]);

const SERVICE_LABEL_MAX = 18;

function isProcessName(column: { key?: string | number; dataIndex?: string | number }): boolean {
  return column.key === "processName" || column.dataIndex === "processName";
}

function shortServiceLabel(label: string): string {
  if (label.length <= SERVICE_LABEL_MAX) {
    return label;
  }
  return `${label.slice(0, SERVICE_LABEL_MAX)}…`;
}

function rowKey(record: PortInfo): string {
  return portKey(record);
}

function customRow(record: PortInfo): {
  class: string;
  onClick: () => void;
  onContextmenu: (event: MouseEvent) => void;
} {
  return {
    class: "port-row",
    onClick: () => {
      if (suppressRowClick.value) {
        suppressRowClick.value = false;
        return;
      }
      emit("detail", record);
    },
    onContextmenu: () => {
      contextRecord.value = record;
      suppressRowClick.value = true;
    },
  };
}

function onHostContextMenu(event: MouseEvent): void {
  const target = event.target;
  if (!(target instanceof Element)) {
    contextRecord.value = null;
    return;
  }
  const row = target.closest("tr[data-row-key]");
  const key = row?.getAttribute("data-row-key");
  contextRecord.value = key
    ? portStore.filteredPorts.find((item) => portKey(item) === key) ?? null
    : null;
  if (contextRecord.value) {
    suppressRowClick.value = true;
  }
}

function onContextOpenChange(open: boolean): void {
  if (open && !contextRecord.value) {
    contextOpen.value = false;
    return;
  }
  contextOpen.value = open;
  if (!open) {
    contextRecord.value = null;
  }
}

function onContextAction(action: RowAction, record: PortInfo): void {
  contextOpen.value = false;
  if (action === "favorite") {
    emit("favorite", record);
    return;
  }
  emit("copy", record);
}

function onKill(record: PortInfo, mode: KillMode): void {
  contextOpen.value = false;
  emit("kill", record, mode);
}

function popupToBody(): HTMLElement {
  return document.body;
}

function onTableChange(page: { current?: number; pageSize?: number }): void {
  currentPage.value = page.current ?? 1;
  pageSize.value = page.pageSize ?? 10;
}

watch(
  () => portStore.keyword,
  () => {
    currentPage.value = 1;
  },
);

watch(
  () => portStore.filteredPorts.length,
  (total) => {
    const maxPage = Math.max(1, Math.ceil(total / pageSize.value) || 1);
    if (currentPage.value > maxPage) {
      currentPage.value = maxPage;
    }
  },
);
</script>

<style scoped>
.table {
  width: 100%;
}

.table :deep(.ant-table-content) {
  overflow-x: hidden;
}

.table :deep(.ant-table table) {
  table-layout: fixed;
}

.table :deep(.port-row:hover > td:first-child) {
  box-shadow: inset 2px 0 0 var(--pk-primary);
}

.actions {
  justify-content: flex-end;
}

.table :deep(.action-btn.ant-btn) {
  width: 30px;
  min-width: 30px;
  height: 30px;
  padding: 0;
  border-radius: 8px;
  color: var(--pk-text-secondary);
}

.table :deep(.action-btn.ant-btn:hover) {
  color: var(--pk-primary);
  background: var(--pk-primary-soft);
}

.table :deep(.ant-pagination) {
  margin: 12px 8px 8px;
}

.table :deep(.port-row) {
  cursor: pointer;
}

.table-context-host {
  width: 100%;
}

.port-cell {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  min-width: 0;
}

.port {
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.service {
  display: inline-block;
  max-width: 100%;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--pk-muted-bg);
  color: var(--pk-text-secondary);
  font-size: 11px;
  font-weight: 500;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: top;
}

.pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 22px;
  padding: 0 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  line-height: 1;
}

.pill-tcp {
  color: var(--pk-primary);
  background: var(--pk-primary-soft);
}

.pill-udp {
  color: var(--pk-purple);
  background: var(--pk-stat-purple-soft);
}

.pill-status {
  color: var(--pk-monitor-on);
  background: var(--pk-success-soft);
}

.pill-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.mono,
.num {
  font-variant-numeric: tabular-nums;
}

.mono {
  color: var(--pk-text-secondary);
  font-family: "Cascadia Code", Consolas, "SF Mono", ui-monospace, monospace;
  font-size: 12px;
}

.num {
  color: var(--pk-text-secondary);
  font-size: 13px;
}

.process-cell {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  max-width: 100%;
}

.process-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.process-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--pk-text) 6%, var(--pk-card));
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.process-icon :deep(svg),
.process-icon :deep(img) {
  width: 16px;
  height: 16px;
  display: block;
}
</style>
