<!--
文件名称：History.vue

文件功能：
操作历史页面，对应原型图历史记录。

主要职责：
- 展示扫描与关闭端口记录，扫描摘要按当前语言显示
- 支持完整分页（总数、每页条数、跳转）与清空

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page">
    <div class="pk-page-header">
      <h1 class="pk-page-title">{{ $t("history.title") }}</h1>
      <a-button danger ghost @click="onClear">{{ $t("history.clear") }}</a-button>
    </div>
    <div class="pk-card table-wrap">
      <a-table
        class="table"
        :columns="columns"
        :data-source="historyStore.items"
        :pagination="pagination"
        row-key="id"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'action'">
            <a-tag :color="isKillAction(record.action) ? 'red' : 'blue'">
              {{ historyActionLabel(record.action) }}
            </a-tag>
          </template>
          <template v-else-if="column.key === 'process'">
            {{ processLabel(record) }}
          </template>
        </template>
      </a-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { message, Modal } from "ant-design-vue";
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useHistoryStore } from "@/stores/history";
import type { HistoryRecord } from "@/types";
import { toErrorMessage } from "@/utils/error";

const historyStore = useHistoryStore();
const { t } = useI18n();
const currentPage = ref(1);
const pageSize = ref(10);

const pagination = computed(() => ({
  current: currentPage.value,
  pageSize: pageSize.value,
  showSizeChanger: true,
  showQuickJumper: true,
  pageSizeOptions: ["10", "20", "50", "100"],
  showTotal: (total: number) => t("history.showTotal", { total }),
}));

const columns = computed(() => [
  { title: t("history.columns.time"), dataIndex: "time", width: 180 },
  { title: t("history.columns.action"), key: "action", dataIndex: "action", width: 120 },
  { title: t("history.columns.port"), dataIndex: "port", width: 90 },
  { title: t("history.columns.protocol"), dataIndex: "protocol", width: 90 },
  { title: t("history.columns.process"), key: "process", dataIndex: "process" },
  { title: t("history.columns.pid"), dataIndex: "pid", width: 100 },
]);

function isKillAction(action: string): boolean {
  return action === "kill" || action === "关闭端口";
}

function historyActionLabel(action: string): string {
  return isKillAction(action) ? t("history.kill") : t("history.scan");
}

/** 扫描记录只存数量；旧数据里的「共 N 个端口」也要能按当前语言显示。 */
function parseScanCount(process: string | null): number | null {
  if (!process) {
    return null;
  }
  const text = process.trim();
  if (/^\d+$/.test(text)) {
    return Number(text);
  }
  const localized = text.match(/(\d+)/);
  if (
    localized &&
    (/个端口/.test(text) || /ports?/i.test(text) || /ポート/.test(text))
  ) {
    return Number(localized[1]);
  }
  return null;
}

function processLabel(record: HistoryRecord): string {
  if (isKillAction(record.action)) {
    const name = record.process?.trim();
    return name && name !== "-" ? name : "-";
  }
  const total = parseScanCount(record.process);
  if (total !== null) {
    return t("history.scanSummary", { total });
  }
  return record.process?.trim() || "-";
}

function onTableChange(page: { current?: number; pageSize?: number }): void {
  currentPage.value = page.current ?? 1;
  pageSize.value = page.pageSize ?? 10;
}

function onClear(): void {
  Modal.confirm({
    title: t("history.clearConfirmTitle"),
    content: t("history.clearConfirm"),
    okText: t("history.clearOk"),
    okType: "danger",
    async onOk() {
      try {
        await historyStore.clear();
        message.success(t("history.cleared"));
      } catch (error) {
        message.error(toErrorMessage(error));
      }
    },
  });
}

onMounted(() => {
  void historyStore.load();
});

watch(
  () => historyStore.items.length,
  (total) => {
    const maxPage = Math.max(1, Math.ceil(total / pageSize.value) || 1);
    if (currentPage.value > maxPage) {
      currentPage.value = maxPage;
    }
  },
);
</script>

<style scoped>
.table-wrap {
  flex: 1;
  overflow: auto;
  padding: 8px 16px 8px 8px;
}

.table :deep(.ant-pagination) {
  margin: 10px 8px 4px;
}
</style>
