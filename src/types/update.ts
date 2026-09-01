/**
 * 文件名称：update.ts
 *
 * 文件功能：
 * 定义应用更新相关前端类型。
 *
 * 主要职责：
 * - 约束更新检查结果
 * - 约束更新流程状态与下载进度
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-25
 */

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "upToDate"
  | "error";

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion: string;
  notes: string;
  date: string | null;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}
