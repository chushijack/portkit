/**
 * 文件名称：main.ts
 *
 * 文件功能：
 * Vue 应用启动入口。
 *
 * 主要职责：
 * - 注册 Pinia、路由、i18n 与 Ant Design Vue
 * - 按窗口标签渲染主界面或快速搜索
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { createPinia } from "pinia";
import { createApp } from "vue";
import Antd from "ant-design-vue";
import App from "./App.vue";
import { i18n } from "@/i18n";
import QuickSearch from "@/views/QuickSearch.vue";
import router from "@/router";
import { setupBrowserDevGuards } from "@/utils/browserDev";
import { getWebviewLabel } from "@/utils/tauri";
import "ant-design-vue/dist/reset.css";
import "@/styles/global.css";

setupBrowserDevGuards();

const pinia = createPinia();
const label = getWebviewLabel();

if (label === "quick-search") {
  const app = createApp(QuickSearch);
  app.use(pinia);
  app.use(i18n);
  app.use(Antd);
  app.mount("#app");
} else {
  const app = createApp(App);
  app.use(pinia);
  app.use(i18n);
  app.use(router);
  app.use(Antd);
  app.mount("#app");
}
