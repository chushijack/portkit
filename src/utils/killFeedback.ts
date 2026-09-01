/**
 * 文件名称：killFeedback.ts
 *
 * 文件功能：
 * 结束进程后根据扫描结果和控制者类型给出明确提示。
 *
 * 主要职责：
 * - 区分没杀掉、被控制者重新拉起、需到控制者处停止、真正释放
 * - 按 launchd / Docker / Supervisor / 服务选择文案
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { message } from "ant-design-vue";
import { i18n } from "@/i18n";
import type { KillOutcome, PortInfo } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { describeKillResult, type KillResultKind } from "@/utils/format";

interface KillOutcomeLabels {
  still: () => string;
  watchdog: () => string;
  success: () => string;
}

function reboundKey(kind: string | undefined): string {
  switch (kind) {
    case "launchDaemon":
    case "launchAgent":
      return "kill.reboundLaunchd";
    case "dockerContainer":
      return "kill.reboundDocker";
    case "windowsService":
      return "kill.reboundService";
    case "supervisor":
      return "kill.rebound";
    default:
      return "kill.rebound";
  }
}

function displayController(value: string | undefined, watchdog: string): string {
  if (!value || value === "-") {
    return watchdog;
  }
  return value;
}

function warnRebound(params: {
  pid: number;
  process: string;
  controller: string;
  controllerKind?: string;
}): void {
  message.warning(
    i18n.global.t(reboundKey(params.controllerKind), {
      pid: params.pid,
      process: params.process,
      controller: params.controller,
    }),
  );
}

/** 扫描结果与后端 KillOutcome 一起判断真实结局。 */
export function notifyKillOutcome(
  ports: PortInfo[],
  port: number,
  originalPid: number,
  processName: string,
  labels: KillOutcomeLabels,
  killError?: unknown,
  killOutcome?: KillOutcome | null,
): KillResultKind {
  if (killOutcome?.kind === "requiresExternalStop") {
    warnRebound({
      pid: originalPid,
      process: killOutcome.process || processName,
      controller: displayController(killOutcome.controller, labels.watchdog()),
      controllerKind: killOutcome.controllerKind,
    });
    return "rebound";
  }
  if (killOutcome?.kind === "requireControllerStop") {
    message.warning(
      i18n.global.t("kill.requireControllerStop", {
        process: killOutcome.process || processName,
        controller: displayController(killOutcome.controller, labels.watchdog()),
      }),
    );
    return "rebound";
  }
  const kind = describeKillResult(ports, port, originalPid);
  const live = ports.find((item) => item.port === port);
  const rebound = kind === "rebound" || killOutcome?.kind === "processRestarted";
  if (rebound) {
    warnRebound({
      pid: live?.pid ?? (killOutcome?.kind === "processRestarted" ? killOutcome.pid : 0),
      process: killOutcome?.process || live?.processName || processName,
      controller: displayController(killOutcome?.controller, labels.watchdog()),
      controllerKind: killOutcome?.controllerKind,
    });
    return "rebound";
  }
  if (kind === "still") {
    message.error(killError ? toErrorMessage(killError) : labels.still());
    return kind;
  }
  message.success(labels.success());
  return "success";
}
