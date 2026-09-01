<!--
文件名称：Favorites.vue

文件功能：
收藏端口页面。

主要职责：
- 自定义添加收藏端口
- 展示收藏列表与占用状态
- 查看进程详情、优雅关闭、强制结束、编辑备注与取消收藏

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="pk-page">
    <div class="pk-page-header">
      <h1 class="pk-page-title">{{ $t("favorites.title") }}</h1>
      <a-button type="primary" @click="openAdd">{{ $t("favorites.add") }}</a-button>
    </div>
    <div class="pk-card table-wrap">
      <a-table
        :columns="columns"
        :data-source="rows"
        :pagination="false"
        :locale="{ emptyText: $t('favorites.empty') }"
        row-key="key"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'processName'">
            <span class="process-cell">
              <ProcessIcon v-if="record.occupied" :name="record.processName" :size="16" />
              <span>{{ record.processName }}</span>
            </span>
          </template>
          <template v-else-if="column.key === 'occupied'">
            <a-tag :color="record.occupied ? 'success' : 'default'">
              {{ record.occupied ? $t("favorites.occupied") : $t("favorites.idle") }}
            </a-tag>
          </template>
          <template v-else-if="column.key === 'actions'">
            <a-space>
              <a-tooltip :title="record.occupied ? $t('port.viewDetail') : $t('favorites.noProcess')">
                <a-button type="text" :disabled="!record.occupied" @click="goDetail(record)">
                  <FaIcon :icon="faEye" :size="15" />
                </a-button>
              </a-tooltip>
              <a-tooltip :title="record.occupied ? $t('port.killGraceful') : $t('favorites.noProcess')">
                <a-button type="text" :disabled="!record.occupied" @click="openKill(record, 'graceful')">
                  <FaIcon :icon="faHand" :size="15" />
                </a-button>
              </a-tooltip>
              <a-tooltip :title="record.occupied ? $t('port.killForce') : $t('favorites.noProcess')">
                <a-button type="text" danger :disabled="!record.occupied" @click="openKill(record, 'force')">
                  <FaIcon :icon="faBolt" :size="15" />
                </a-button>
              </a-tooltip>
              <a-tooltip :title="$t('favorites.note')">
                <a-button type="text" @click="edit(record)">
                  <FaIcon :icon="faPenToSquare" :size="15" />
                </a-button>
              </a-tooltip>
              <a-tooltip :title="$t('port.unfavorite')">
                <a-button type="text" danger @click="remove(record)">
                  <FaIcon :icon="faStar" :size="15" />
                </a-button>
              </a-tooltip>
            </a-space>
          </template>
        </template>
      </a-table>
    </div>

    <a-modal
      v-model:open="formOpen"
      :title="editing ? $t('favorites.editNote') : $t('favorites.addTitle')"
      :confirm-loading="saving"
      :ok-text="$t('favorites.save')"
      :cancel-text="$t('favorites.cancel')"
      @ok="saveForm"
    >
      <a-form layout="vertical">
        <a-form-item :label="$t('favorites.port')" required>
          <a-input-number
            v-model:value="form.port"
            :min="1"
            :max="65535"
            :disabled="Boolean(editing)"
            style="width: 100%"
            :placeholder="$t('favorites.portPlaceholder')"
          />
        </a-form-item>
        <a-form-item :label="$t('favorites.protocol')" required>
          <a-radio-group v-model:value="form.protocol" :disabled="Boolean(editing)">
            <a-radio value="TCP">TCP</a-radio>
            <a-radio value="UDP">UDP</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item :label="$t('favorites.note')">
          <a-input v-model:value="form.note" :placeholder="$t('favorites.notePlaceholder')" />
        </a-form-item>
      </a-form>
    </a-modal>

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
import { faBolt, faEye, faHand, faPenToSquare, faStar } from "@fortawesome/free-solid-svg-icons";
import { message } from "ant-design-vue";
import { computed, onMounted, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import FaIcon from "@/components/FaIcon.vue";
import KillConfirmModal from "@/components/KillConfirmModal.vue";
import ProcessIcon from "@/components/ProcessIcon.vue";
import { scanPorts } from "@/composables/useAutoRefresh";
import { useFavoriteStore } from "@/stores/favorite";
import { usePortStore } from "@/stores/port";
import { useProcessStore } from "@/stores/process";
import type { FavoritePort, KillMode, KillOutcome } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { notifyKillOutcome } from "@/utils/killFeedback";
import { displayText } from "@/utils/processIcon";

interface FavoriteRow extends FavoritePort {
  key: string;
  occupied: boolean;
  processName: string;
  pid: number | null;
}

interface FavoriteForm {
  port: number | null;
  protocol: "TCP" | "UDP";
  note: string;
}

const favoriteStore = useFavoriteStore();
const portStore = usePortStore();
const processStore = useProcessStore();
const router = useRouter();
const { t } = useI18n();
const formOpen = ref(false);
const saving = ref(false);
const editing = ref<FavoriteRow | null>(null);
const killOpen = ref(false);
const killing = ref(false);
const pending = ref<FavoriteRow | null>(null);
const pendingMode = ref<KillMode>("force");
const form = reactive<FavoriteForm>({
  port: null,
  protocol: "TCP",
  note: "",
});

const columns = computed(() => [
  { title: t("favorites.columns.port"), dataIndex: "port", width: 100 },
  { title: t("favorites.columns.protocol"), dataIndex: "protocol", width: 90 },
  { title: t("favorites.columns.note"), dataIndex: "note" },
  { title: t("favorites.columns.processName"), key: "processName", dataIndex: "processName", width: 176 },
  { title: t("favorites.columns.occupied"), key: "occupied", width: 100 },
  { title: t("favorites.columns.actions"), key: "actions", width: 212, align: "center" as const },
]);

const rows = computed<FavoriteRow[]>(() =>
  favoriteStore.items.map((item) => {
    const live =
      portStore.ports.find(
        (port) => port.port === item.port && port.protocol === item.protocol,
      ) ?? portStore.ports.find((port) => port.port === item.port);
    return {
      ...item,
      key: String(item.port),
      occupied: Boolean(live),
      processName: displayText(live?.processName),
      pid: live?.pid ?? null,
    };
  }),
);

function resetForm(): void {
  form.port = null;
  form.protocol = "TCP";
  form.note = "";
}

function openAdd(): void {
  editing.value = null;
  resetForm();
  formOpen.value = true;
}

function edit(record: FavoriteRow): void {
  editing.value = record;
  form.port = record.port;
  form.protocol = record.protocol === "UDP" ? "UDP" : "TCP";
  form.note = record.note;
  formOpen.value = true;
}

function validateForm(): string | null {
  if (form.port === null || form.port < 1 || form.port > 65535) {
    return t("favorites.invalidPort");
  }
  if (!editing.value && favoriteStore.isFavorite(form.port)) {
    return t("favorites.alreadyExists");
  }
  return null;
}

async function saveForm(): Promise<void> {
  const errorText = validateForm();
  if (errorText || form.port === null) {
    message.warning(errorText ?? t("favorites.invalidPort"));
    return Promise.reject(new Error(errorText ?? "invalid port"));
  }

  saving.value = true;
  try {
    if (editing.value) {
      await favoriteStore.updateNote(form.port, form.note.trim());
      message.success(t("favorites.noteUpdated"));
    } else {
      await favoriteStore.add(form.port, form.protocol, form.note.trim());
      message.success(t("favorites.added"));
    }
    formOpen.value = false;
  } catch (error) {
    message.error(toErrorMessage(error));
  } finally {
    saving.value = false;
  }
}

async function remove(record: FavoriteRow): Promise<void> {
  try {
    await favoriteStore.remove(record.port);
    message.success(t("favorites.removed"));
  } catch (error) {
    message.error(toErrorMessage(error));
  }
}

function goDetail(record: FavoriteRow): void {
  if (!record.occupied || record.pid === null) {
    message.warning(t("favorites.noProcess"));
    return;
  }
  void router.push({
    name: "process",
    params: { pid: String(record.pid) },
    query: { port: String(record.port), protocol: record.protocol },
  });
}

function openKill(record: FavoriteRow, mode: KillMode): void {
  if (!record.occupied || record.pid === null) {
    message.warning(t("favorites.noProcess"));
    return;
  }
  pending.value = record;
  pendingMode.value = mode;
  killOpen.value = true;
}

async function confirmKill(): Promise<void> {
  if (!pending.value || pending.value.pid === null) {
    message.warning(t("port.noProcess"));
    return;
  }
  const target = pending.value;
  const pid = target.pid;
  if (pid === null) {
    message.warning(t("port.noProcess"));
    return;
  }
  killing.value = true;
  let killError: unknown;
  let killOutcome: KillOutcome | null = null;
  try {
    killOutcome = await processStore.kill({
      pid,
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
      pid,
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

onMounted(() => {
  void favoriteStore.load();
  if (portStore.ports.length === 0) {
    void scanPorts({ recordHistory: false });
  }
});
</script>

<style scoped>
.table-wrap {
  flex: 1;
  overflow: auto;
  padding: 8px 16px 8px 8px;
}

.process-cell {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
</style>
