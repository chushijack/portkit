/**
 * 文件名称：types.ts
 *
 * 文件功能：
 * i18n 文案类型，以简体中文为结构基准。
 *
 * 主要职责：
 * - 约束英文、日文与中文键一致
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import type zhCN from "./locales/zh-CN";

export type MessageSchema = typeof zhCN;
