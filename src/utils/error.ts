/**
 * 文件名称：error.ts
 *
 * 文件功能：
 * 把未知错误转成可展示文案。
 *
 * 主要职责：
 * - 识别后端错误码并走 i18n
 * - 统一页面 message.error 的入参
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { i18n } from "@/i18n";

const ERROR_CODES = [
  "PROCESS_NOT_FOUND",
  "PERMISSION_DENIED",
  "PROCESS_STILL_RUNNING",
  "PROCESS_RESTARTED",
  "PROTECTED_PROCESS",
  "SELF_PROCESS",
  "INVALID_SCAN_RANGE",
  "INVALID_PORT",
  "INVALID_PROTOCOL",
  "FAVORITE_ALREADY_EXISTS",
  "FAVORITE_NOT_FOUND",
  "UNSUPPORTED_PLATFORM",
  "DATA_DIR",
  "STORAGE",
  "SYSTEM_ERROR",
  "UPDATE_CHECK_FAILED",
  "UPDATE_DOWNLOAD_FAILED",
  "UPDATE_INSTALL_FAILED",
  "UPDATE_NOT_AVAILABLE",
] as const;

type ErrorCode = (typeof ERROR_CODES)[number];

function isErrorCode(value: string): value is ErrorCode {
  return (ERROR_CODES as readonly string[]).includes(value);
}

function extractRaw(error: unknown): string {
  if (typeof error === "string" && error.length > 0) {
    return error;
  }
  if (error instanceof Error && error.message) {
    return error.message;
  }
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message: unknown }).message;
    if (typeof message === "string" && message.length > 0) {
      return message;
    }
  }
  return "";
}

export interface RestartedError {
  process: string;
  restarter: string;
  pid: number;
}

/** 解析后端带回的进程名、拉起方和新 PID。 */
export function parseRestartedError(error: unknown): RestartedError | null {
  const raw = extractRaw(error);
  const parts = raw.split("|");
  if (parts[0] !== "PROCESS_RESTARTED" || parts.length < 4) {
    return null;
  }
  return {
    process: parts[1],
    restarter: parts[2],
    pid: Number(parts[3]) || 0,
  };
}

/** 把 Tauri / 业务错误转成当前语言的提示。 */
export function toErrorMessage(error: unknown): string {
  const t = i18n.global.t;
  const raw = extractRaw(error);
  if (!raw) {
    return t("errors.UNKNOWN");
  }
  const restarted = parseRestartedError(error);
  if (restarted) {
    const restarter =
      !restarted.restarter || restarted.restarter === "-"
        ? t("errors.watchdog")
        : restarted.restarter;
    return t("errors.PROCESS_RESTARTED", {
      process: restarted.process,
      controller: restarter,
      pid: restarted.pid,
    });
  }
  const separator = raw.indexOf("|");
  if (separator > 0) {
    const code = raw.slice(0, separator);
    const detail = raw.slice(separator + 1);
    if (isErrorCode(code)) {
      return t(`errors.${code}`, { detail });
    }
  }
  if (isErrorCode(raw)) {
    return t(`errors.${raw}`);
  }
  return raw;
}
