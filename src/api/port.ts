/**
 * 文件名称：port.ts
 *
 * 文件功能：
 * 封装端口扫描相关 Tauri 调用。
 *
 * 主要职责：
 * - 扫描端口
 * - 获取常用端口预设
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { invoke } from "@/api/invoke";
import type { CommonPort, PortInfo, ScanRequest } from "@/types";

export const portApi = {
  scanPorts(request: ScanRequest): Promise<PortInfo[]> {
    return invoke<PortInfo[]>("scan_ports", { request });
  },
  getCommonPorts(): Promise<CommonPort[]> {
    return invoke<CommonPort[]>("get_common_ports");
  },
};
