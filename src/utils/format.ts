/**
 * 文件名称：format.ts
 *
 * 文件功能：
 * 提供前端展示用的格式化函数。
 *
 * 主要职责：
 * - 格式化内存占用
 * - 生成端口唯一键
 * - 结束进程后根据扫描结果给出提示
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import type { PortInfo } from "@/types";

/** 把字节数格式化为 MB 展示。 */
export function formatMemory(bytes: number): string {
  const megaBytes = bytes / (1024 * 1024);
  return `${megaBytes.toFixed(1)} MB`;
}

/** 从 `ip:port` 或 `[ipv6]:port` 中取出监听地址。 */
export function listenHost(address: string): string {
  if (address.startsWith("[")) {
    const end = address.indexOf("]");
    return end > 0 ? address.slice(1, end) : address;
  }
  const index = address.lastIndexOf(":");
  return index === -1 ? address : address.slice(0, index);
}

/** 端口+协议+PID，用于列表 diff 与表格行 key。 */
export function portKey(port: Pick<PortInfo, "port" | "protocol" | "pid">): string {
  return `${port.protocol}:${port.port}:${port.pid}`;
}

/** 修饰修饰键文案，Windows 显示 Ctrl，macOS 显示 ⌘。 */
export function modifierLabel(isMac: boolean): string {
  return isMac ? "⌘" : "Ctrl";
}

/** 结束进程并重新扫描后，区分仍存活、被拉起和真正释放。 */
export type KillResultKind = "still" | "rebound" | "success";

export function describeKillResult(
  ports: PortInfo[],
  port: number,
  pid: number,
): KillResultKind {
  const live = ports.find((item) => item.port === port);
  if (live?.pid === pid) {
    return "still";
  }
  if (live) {
    return "rebound";
  }
  return "success";
}
