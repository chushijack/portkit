/**
 * 文件名称：index.ts
 *
 * 文件功能：
 * 统一导出桌面端 API。
 *
 * 主要职责：
 * - 聚合端口、进程、收藏、历史、设置、更新与窗口接口
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export { portApi } from "./port";
export { processApi } from "./process";
export { favoriteApi } from "./favorite";
export { historyApi } from "./history";
export { settingsApi } from "./settings";
export { releaseApi } from "./release";
export { updateApi } from "./update";
export { windowApi } from "./window";
