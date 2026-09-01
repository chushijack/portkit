<!--
文件名称：UpdateDialog.vue

文件功能：
应用更新弹窗。

主要职责：
- 展示最新版本与更新日志
- 显示下载进度
- 触发下载安装并重启
- 支持关闭弹窗后在后台继续更新

作者：Chushi Jack
创建时间：2026-08-25
-->
<template>
  <a-modal
    :open="updateStore.dialogOpen"
    :title="$t('update.title')"
    :confirm-loading="installing"
    :ok-text="$t('update.now')"
    :cancel-text="installing ? $t('update.background') : $t('update.later')"
    :ok-button-props="{ disabled: installing }"
    :mask-closable="!installing"
    :closable="true"
    :keyboard="!installing"
    @ok="onInstall"
    @cancel="onCancel"
  >
    <div class="meta">
      <div>
        <span>{{ $t("update.current") }}</span>
        {{ info?.currentVersion ?? updateStore.currentVersion }}
      </div>
      <div>
        <span>{{ $t("update.latest") }}</span>
        {{ info?.latestVersion ?? "-" }}
      </div>
    </div>

    <div class="notes">
      <div class="notes-title">{{ $t("update.notes") }}</div>
      <pre class="notes-body">{{ notesText }}</pre>
    </div>

    <div v-if="installing" class="progress">
      <a-progress
        :percent="updateStore.status === 'installing' ? 100 : updateStore.progressPercent"
        :status="updateStore.status === 'installing' ? 'success' : 'active'"
        :show-info="Boolean(updateStore.totalBytes)"
      />
      <div class="pk-muted">
        {{ updateStore.status === "installing" ? $t("update.installing") : $t("update.downloading") }}
      </div>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { message } from "ant-design-vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useUpdateStore } from "@/stores/update";
import { toErrorMessage } from "@/utils/error";

const updateStore = useUpdateStore();
const { t } = useI18n();

const info = computed(() => updateStore.info);
const installing = computed(
  () => updateStore.status === "downloading" || updateStore.status === "installing",
);
const notesText = computed(() => {
  const notes = info.value?.notes.trim() ?? "";
  return notes.length > 0 ? notes : t("update.noNotes");
});

function onCancel(): void {
  if (installing.value) {
    updateStore.moveToBackground();
    return;
  }
  updateStore.closeDialog();
}

async function onInstall(): Promise<void> {
  try {
    await updateStore.install();
  } catch (error) {
    message.error(toErrorMessage(error));
    throw error;
  }
}
</script>

<style scoped>
.meta {
  display: grid;
  gap: 8px;
}

.meta span {
  display: inline-block;
  width: 72px;
  color: var(--pk-text-secondary);
}

.notes {
  margin-top: 16px;
}

.notes-title {
  color: var(--pk-text-secondary);
  margin-bottom: 8px;
}

.notes-body {
  margin: 0;
  max-height: 220px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--pk-muted-bg);
  color: var(--pk-text);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  font-size: 13px;
  line-height: 1.6;
}

.progress {
  margin-top: 16px;
}
</style>
