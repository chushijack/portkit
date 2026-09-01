/**
 * 文件名称：process.ts
 *
 * 文件功能：
 * 定义进程详情页数据结构。
 *
 * 主要职责：
 * - 约束进程元数据与资源占用字段
 * - 约束释放端口的业务结果
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export interface ProcessDetail {
  pid: number;
  name: string;
  displayName: string;
  serviceLabels: string[];
  status: string;
  user: string;
  startTime: string;
  duration: string;
  priority: string;
  path: string;
  command: string;
  cpuUsage: number;
  memoryBytes: number;
  ports: number[];
}

export type KillMode = "graceful" | "force";

export type ControllerKind =
  | "windowsService"
  | "launchDaemon"
  | "launchAgent"
  | "application"
  | "supervisor"
  | "dockerContainer"
  | "unknown";

/** 后端释放端口的结果：成功、被控制者重新拉起、或需到控制者处停止。 */
export type KillOutcome =
  | {
      kind: "released";
      process: string;
      controller: string;
      controllerKind: string;
    }
  | {
      kind: "processRestarted";
      process: string;
      controller: string;
      controllerKind: string;
      pid: number;
    }
  | {
      kind: "requiresExternalStop";
      process: string;
      controller: string;
      controllerKind: string;
    }
  | {
      kind: "requireControllerStop";
      process: string;
      controller: string;
      controllerKind: string;
    };
