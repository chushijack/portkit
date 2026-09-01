/**
 * 文件名称：history.ts
 *
 * 文件功能：
 * 定义操作历史数据结构。
 *
 * 主要职责：
 * - 约束历史表格字段
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export interface HistoryRecord {
  id: string;
  time: string;
  action: string;
  port: number | null;
  protocol: string | null;
  process: string | null;
  pid: number | null;
}
