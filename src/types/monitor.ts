/**
 * 文件名称：monitor.ts
 *
 * 文件功能：
 * 定义实时监控变更事件类型。
 *
 * 主要职责：
 * - 约束新增/关闭端口提醒数据
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export type MonitorChangeType = "appeared" | "closed";

export interface MonitorEvent {
  id: string;
  time: string;
  type: MonitorChangeType;
  port: number;
  protocol: string;
  processName: string;
  pid: number;
}
