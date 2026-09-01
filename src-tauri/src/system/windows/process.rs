//! 文件名称：process.rs
//!
//! 文件功能：
//! Windows 基础进程操作：查询存活、结束进程、优先级、打开目录。
//!
//! 主要职责：
//! - 优雅关闭：控制台进程发 Ctrl+C，窗口进程发 WM_CLOSE
//! - 强制结束用 TerminateProcess / taskkill /F
//! - 用进程句柄判断 PID 是否仍在运行
//! - 用进程令牌 SID 反查账户名
//! - sysinfo 读不到启动时间时用 GetProcessTimes 补一次
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::Security::{
    GetTokenInformation, LookupAccountSidW, TokenUser, PSID, SID_NAME_USE, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Console::{
    AttachConsole, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler, CTRL_C_EVENT,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetPriorityClass, GetProcessTimes, OpenProcess, OpenProcessToken,
    TerminateProcess, ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS,
    HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, REALTIME_PRIORITY_CLASS,
};

use crate::errors::AppError;
use crate::models::KillMode;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const STILL_ACTIVE: u32 = 259;
const ACCESS_DENIED: u32 = 5;
const ACCESS_DENIED_HRESULT: u32 = 0x8007_0005;

/// 发出结束请求后立即返回。是否已经退出由服务层观察，避免每个 PID 空等 5 秒。
pub fn request_kill(pid: u32, mode: KillMode) {
    match mode {
        KillMode::Graceful => {
            // java / node 等控制台进程没有窗口，WM_CLOSE 会被忽略，必须先发 Ctrl+C。
            let _ = try_console_ctrl_c(pid);
            if process_is_running(pid) {
                let _ = try_taskkill_graceful(pid);
            }
        }
        KillMode::Force => {
            let _ = try_terminate(pid);
            if process_is_running(pid) {
                let _ = try_taskkill_force(pid);
            }
        }
    }
}

/// 用进程句柄判断 PID 是否仍在运行。
///
/// OpenProcess 被拒绝说明进程还在，不能当成已经退出。
pub(crate) fn process_is_running(pid: u32) -> bool {
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => {
                let mut code = 0_u32;
                let running = GetExitCodeProcess(handle, &mut code).is_ok() && code == STILL_ACTIVE;
                let _ = CloseHandle(handle);
                running
            }
            Err(error) => is_access_denied(&error),
        }
    }
}

fn is_access_denied(error: &windows::core::Error) -> bool {
    let code = error.code().0 as u32;
    code == ACCESS_DENIED || code == ACCESS_DENIED_HRESULT
}

fn try_terminate(pid: u32) -> bool {
    unsafe {
        let Ok(handle) = OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        ) else {
            return false;
        };
        let terminated = TerminateProcess(handle, 1).is_ok();
        let mut code = 0_u32;
        let exited = GetExitCodeProcess(handle, &mut code).is_ok() && code != STILL_ACTIVE;
        let _ = CloseHandle(handle);
        terminated || exited
    }
}

/// 把 PortKit 临时挂到目标控制台，发送 Ctrl+C，让 Java / Node 等走关闭钩子。
///
/// `taskkill` 不带 /F 只发 WM_CLOSE，控制台程序收不到，所以必须走这条路径。
/// 发送前忽略本进程的 Ctrl+C，避免 PortKit 把自己结束掉。
fn try_console_ctrl_c(pid: u32) -> bool {
    unsafe {
        let _ = FreeConsole();
        if AttachConsole(pid).is_err() {
            return false;
        }
        let _ = SetConsoleCtrlHandler(None, true);
        let sent = GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0).is_ok();
        std::thread::sleep(Duration::from_millis(200));
        let _ = FreeConsole();
        let _ = SetConsoleCtrlHandler(None, false);
        sent
    }
}

/// 不带 /F：给窗口进程发 WM_CLOSE，给进程自行收尾的机会。
fn try_taskkill_graceful(pid: u32) -> bool {
    let pid_text = pid.to_string();
    let closed = Command::new("taskkill")
        .args(["/PID", &pid_text])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if closed && !process_is_running(pid) {
        return true;
    }
    Command::new("taskkill")
        .args(["/PID", &pid_text, "/T"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn try_taskkill_force(pid: u32) -> bool {
    let pid_text = pid.to_string();
    let force = Command::new("taskkill")
        .args(["/PID", &pid_text, "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if force && !process_is_running(pid) {
        return true;
    }
    Command::new("taskkill")
        .args(["/PID", &pid_text, "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 读取进程优先级，返回前端可翻译的级别码。
pub fn process_priority(pid: u32) -> String {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return "normal".to_string();
        };
        let class = PROCESS_CREATION_FLAGS(GetPriorityClass(handle));
        let _ = CloseHandle(handle);
        match class {
            IDLE_PRIORITY_CLASS => "idle".to_string(),
            BELOW_NORMAL_PRIORITY_CLASS => "belowNormal".to_string(),
            ABOVE_NORMAL_PRIORITY_CLASS => "aboveNormal".to_string(),
            HIGH_PRIORITY_CLASS => "high".to_string(),
            REALTIME_PRIORITY_CLASS => "realtime".to_string(),
            NORMAL_PRIORITY_CLASS => "normal".to_string(),
            _ => "normal".to_string(),
        }
    }
}

/// 通过进程令牌 SID 反查账户名。
///
/// sysinfo 的用户列表只有本地账户，SYSTEM / 服务账户会对不上，所以这里单独查。
pub(crate) fn process_account_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut token = HANDLE::default();
        let opened = OpenProcessToken(handle, TOKEN_QUERY, &mut token);
        let _ = CloseHandle(handle);
        opened.ok()?;

        let mut size = 0_u32;
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut size);
        if size == 0 {
            let _ = CloseHandle(token);
            return None;
        }

        let mut buffer = vec![0_u8; size as usize];
        let queried = GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        );
        let _ = CloseHandle(token);
        queried.ok()?;

        let token_user = buffer.as_ptr().cast::<TOKEN_USER>().read_unaligned();
        lookup_sid_name(token_user.User.Sid)
    }
}

/// sysinfo 对受保护进程常返回启动时间 0，改用创建时间换算 Unix 秒。
pub(crate) fn process_start_unix(pid: u32) -> Option<u64> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let queried = GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user);
        let _ = CloseHandle(handle);
        queried.ok()?;
        filetime_to_unix(&created)
    }
}

fn filetime_to_unix(time: &FILETIME) -> Option<u64> {
    let ticks = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
    const UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
    if ticks <= UNIX_EPOCH_TICKS {
        return None;
    }
    Some((ticks - UNIX_EPOCH_TICKS) / 10_000_000)
}

fn lookup_sid_name(sid: PSID) -> Option<String> {
    unsafe {
        let mut name_len = 0_u32;
        let mut domain_len = 0_u32;
        let mut usage = SID_NAME_USE::default();
        let _ = LookupAccountSidW(
            PCWSTR::null(),
            sid,
            PWSTR::null(),
            &mut name_len,
            PWSTR::null(),
            &mut domain_len,
            &mut usage,
        );
        if name_len == 0 {
            return None;
        }

        let mut name = vec![0_u16; name_len as usize];
        let mut domain = vec![0_u16; domain_len as usize];
        LookupAccountSidW(
            PCWSTR::null(),
            sid,
            PWSTR(name.as_mut_ptr()),
            &mut name_len,
            PWSTR(domain.as_mut_ptr()),
            &mut domain_len,
            &mut usage,
        )
        .ok()?;

        let account = utf16_lossy(&name, name_len);
        let domain_name = utf16_lossy(&domain, domain_len);
        if domain_name.is_empty() {
            Some(account)
        } else {
            Some(format!("{domain_name}\\{account}"))
        }
    }
}

fn utf16_lossy(buffer: &[u16], len: u32) -> String {
    let end = (len as usize).min(buffer.len());
    let slice = &buffer[..end];
    let trimmed = slice.strip_suffix(&[0]).unwrap_or(slice);
    String::from_utf16_lossy(trimmed)
}

/// 打开进程所在目录。传入文件路径时打开父目录。
pub fn open_directory(path: &str) -> Result<(), AppError> {
    if path.is_empty() {
        return Err(AppError::SystemError("进程路径为空".to_string()));
    }

    let target = Path::new(path);
    let dir = if target.is_dir() {
        target
    } else {
        target.parent().unwrap_or(target)
    };

    Command::new("explorer")
        .arg(dir.as_os_str())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::filetime_to_unix;
    use windows::Win32::Foundation::FILETIME;

    fn filetime_from_ticks(ticks: u64) -> FILETIME {
        FILETIME {
            dwLowDateTime: ticks as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        }
    }

    #[test]
    fn filetime_before_unix_epoch_is_unknown() {
        assert_eq!(filetime_to_unix(&filetime_from_ticks(0)), None);
        assert_eq!(
            filetime_to_unix(&filetime_from_ticks(116_444_736_000_000_000)),
            None
        );
    }

    #[test]
    fn filetime_after_unix_epoch_converts() {
        let ticks = 116_444_736_000_000_000 + 1_700_000_000 * 10_000_000;
        assert_eq!(filetime_to_unix(&filetime_from_ticks(ticks)), Some(1_700_000_000));
    }
}
