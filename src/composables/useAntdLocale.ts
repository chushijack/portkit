/**
 * 文件名称：useAntdLocale.ts
 *
 * 文件功能：
 * 按当前界面语言提供 Ant Design Vue 的 locale。
 *
 * 主要职责：
 * - 映射中/英/日到 antd 内置语言包
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { computed, type ComputedRef } from "vue";
import type { Locale } from "ant-design-vue/es/locale";
import enUS from "ant-design-vue/es/locale/en_US";
import jaJP from "ant-design-vue/es/locale/ja_JP";
import zhCN from "ant-design-vue/es/locale/zh_CN";
import { useSettingsStore } from "@/stores/settings";

/** 分页、空状态等组件文案跟随 PortKit 语言。 */
export function useAntdLocale(): ComputedRef<Locale> {
  const settingsStore = useSettingsStore();
  return computed(() => {
    switch (settingsStore.resolvedLocale) {
      case "ja":
        return jaJP;
      case "en":
        return enUS;
      default:
        return zhCN;
    }
  });
}
