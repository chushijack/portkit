/**
 * 文件名称：vite.config.ts
 *
 * 文件功能：
 * Vite 开发与构建配置，适配 Tauri 固定端口。
 *
 * 主要职责：
 * - 配置 Vue 插件与路径别名
 * - 固定开发服务器端口 1420
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
