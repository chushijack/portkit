<!--
文件名称：UpdateBackgroundBanner.vue

文件功能：
更新下载/安装时的后台进度条。

主要职责：
- 在关闭更新弹窗后继续展示进度
- 允许重新打开更新详情

作者：Chushi Jack
创建时间：2026-08-29
-->
<template>
  <div v-if="updateStore.backgroundUpdating" class="update-banner">
    <div class="update-banner__content">
      <div class="update-banner__text">
        <span class="update-banner__title">{{ $t("update.backgroundTitle") }}</span>
        <span class="pk-muted">{{ statusText }}</span>
      </div>
      <a-progress
        class="update-banner__progress"
        :percent="progressPercent"
        :status="updateStore.status === 'installing' ? 'success' : 'active'"
        :show-info="Boolean(updateStore.totalBytes)"
        size="small"
      />
      <a-button type="link" size="small" @click="updateStore.openDialog()">
        {{ $t("update.viewDetails") }}
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useUpdateStore } from "@/stores/update";

const updateStore = useUpdateStore();
const { t } = useI18n();

const progressPercent = computed(() =>
  updateStore.status === "installing" ? 100 : updateStore.progressPercent,
);

const statusText = computed(() =>
  updateStore.status === "installing" ? t("update.installing") : t("update.downloading"),
);
</script>

<style scoped>
.update-banner {
  padding: 10px 20px;
  background: var(--pk-card);
  border-bottom: 1px solid var(--pk-border);
}

.update-banner__content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.update-banner__text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 180px;
}

.update-banner__title {
  font-size: 13px;
  font-weight: 600;
}

.update-banner__progress {
  flex: 1;
  min-width: 160px;
  margin: 0;
}
</style>
