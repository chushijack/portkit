<!--
文件名称：KillConfirmModal.vue

文件功能：
结束进程二次确认弹窗。

主要职责：
- 按优雅关闭 / 强制结束展示不同风险提示
- 展示端口、进程、PID

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <a-modal
    :open="open"
    :title="mode === 'force' ? $t('kill.titleForce') : $t('kill.titleGraceful')"
    :confirm-loading="loading"
    :ok-text="mode === 'force' ? $t('kill.okForce') : $t('kill.okGraceful')"
    :cancel-text="$t('kill.cancel')"
    :ok-type="mode === 'force' ? 'danger' : 'primary'"
    @ok="emit('confirm')"
    @cancel="emit('update:open', false)"
    :mask-closable="false"
  >
    <div class="meta">
      <div>
        <span>{{ $t("kill.mode") }}</span>
        {{ mode === "force" ? $t("kill.modeForce") : $t("kill.modeGraceful") }}
      </div>
      <div><span>{{ $t("kill.port") }}</span>{{ port ?? "-" }}</div>
      <div><span>{{ $t("kill.processName") }}</span>{{ processName ?? "-" }}</div>
      <div><span>{{ $t("kill.pid") }}</span>{{ pid ?? "-" }}</div>
    </div>
    <div class="warn">
      <FaIcon :icon="mode === 'force' ? faBolt : faHand" :size="18" />
      {{ mode === "force" ? $t("kill.warnForce") : $t("kill.warnGraceful") }}
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { faBolt, faHand } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "@/components/FaIcon.vue";
import type { KillMode } from "@/types";

withDefaults(
  defineProps<{
    open: boolean;
    loading?: boolean;
    mode?: KillMode;
    port?: number | null;
    processName?: string | null;
    pid?: number | null;
  }>(),
  {
    mode: "force",
  },
);

const emit = defineEmits<{
  "update:open": [value: boolean];
  confirm: [];
}>();
</script>

<style scoped>
.warn {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  color: var(--pk-warning-text);
  background: var(--pk-warning-bg);
  border-radius: 8px;
  padding: 10px 12px;
  margin-top: 16px;
}

.meta {
  display: grid;
  gap: 8px;
}

.meta span {
  display: inline-block;
  width: 72px;
  color: var(--pk-text-secondary);
}
</style>
