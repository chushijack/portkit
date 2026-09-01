/**
 * 文件名称：index.ts
 *
 * 文件功能：
 * 桌面端路由表。
 *
 * 主要职责：
 * - 注册端口、监控、收藏、历史、设置与进程详情页
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { createRouter, createWebHistory } from "vue-router";
import PortList from "@/views/PortList.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "ports", component: PortList },
    {
      path: "/monitor",
      name: "monitor",
      component: () => import("@/views/Monitor.vue"),
    },
    {
      path: "/favorites",
      name: "favorites",
      component: () => import("@/views/Favorites.vue"),
    },
    {
      path: "/history",
      name: "history",
      component: () => import("@/views/History.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/Settings.vue"),
    },
    {
      path: "/process/:pid",
      name: "process",
      component: () => import("@/views/ProcessDetail.vue"),
      props: true,
    },
  ],
});

export default router;
