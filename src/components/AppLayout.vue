<!--
文件名称：AppLayout.vue

文件功能：
主窗口页面骨架，组合侧边栏、顶栏与内容区。

主要职责：
- 承载路由页面
- 转发刷新事件

作者：Chushi Jack
创建时间：2026-08-20
-->
<template>
  <div class="layout">
    <AppSidebar />
    <section class="main">
      <AppHeader @refresh="emit('refresh')" />
      <UpdateBackgroundBanner />
      <div class="content">
        <RouterView />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import AppHeader from "@/components/AppHeader.vue";
import AppSidebar from "@/components/AppSidebar.vue";
import UpdateBackgroundBanner from "@/components/UpdateBackgroundBanner.vue";

const emit = defineEmits<{ refresh: [] }>();
</script>

<style scoped>
.layout {
  display: flex;
  width: 100%;
  height: 100%;
  background: var(--pk-bg);
}

.main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
  min-height: 0;
  padding: 20px;
  overflow: hidden;
}
</style>
