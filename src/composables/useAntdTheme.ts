/**
 * 文件名称：useAntdTheme.ts
 *
 * 文件功能：
 * 根据当前亮暗色生成 Ant Design Vue 主题配置。
 *
 * 主要职责：
 * - 统一主窗口与快速搜索窗口的组件主题
 * - 让表格在深色模式下有足够对比度
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { computed, type ComputedRef } from "vue";
import { theme } from "ant-design-vue";
import { useSettingsStore } from "@/stores/settings";

interface PortKitTheme {
  algorithm: typeof theme.defaultAlgorithm | typeof theme.darkAlgorithm;
  token: {
    colorPrimary: string;
    borderRadius: number;
    fontFamily: string;
    colorBgContainer: string;
    colorBorderSecondary: string;
    colorText: string;
    colorTextSecondary: string;
  };
  components: {
    Table: {
      headerBg: string;
      headerColor: string;
      rowHoverBg: string;
      borderColor: string;
      headerSplitColor: string;
    };
  };
}

const FONT = '"Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif';

/** 返回跟随 PortKit 亮暗色的 Ant Design 主题。 */
export function useAntdTheme(): ComputedRef<PortKitTheme> {
  const settingsStore = useSettingsStore();
  return computed(() => {
    const dark = settingsStore.resolvedTheme === "dark";
    return {
      algorithm: dark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      token: {
        colorPrimary: "#3d7eff",
        borderRadius: 10,
        fontFamily: FONT,
        colorBgContainer: dark ? "#1e293b" : "#ffffff",
        colorBorderSecondary: dark ? "#475569" : "#e7ebf2",
        colorText: dark ? "#e5e7eb" : "#1f2937",
        colorTextSecondary: dark ? "#94a3b8" : "#6b7280",
      },
      components: {
        Table: {
          headerBg: dark ? "#273548" : "#f8fafc",
          headerColor: dark ? "#cbd5e1" : "#64748b",
          rowHoverBg: dark ? "#334155" : "#eef3fb",
          borderColor: dark ? "#475569" : "#e7ebf2",
          headerSplitColor: dark ? "#475569" : "#e7ebf2",
        },
      },
    };
  });
}
