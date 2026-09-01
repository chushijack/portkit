/**
 * 文件名称：port.ts
 *
 * 文件功能：
 * 定义端口扫描与列表展示相关类型。
 *
 * 主要职责：
 * - 约束扫描请求
 * - 约束端口表格行数据
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export type ScanMode = "all" | "common" | "custom";

export interface ScanRequest {
  mode: ScanMode;
  startPort?: number;
  endPort?: number;
  recordHistory?: boolean;
}

export interface CommonPort {
  port: number;
  label: string;
}

export interface PortInfo {
  port: number;
  protocol: string;
  status: string;
  listenAddress: string;
  processName: string;
  pid: number;
  occupiedTime: string;
  occupiedSeconds: number;
  startTime: number;
  serviceLabel: string | null;
  command: string | null;
}
