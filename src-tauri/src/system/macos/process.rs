//! 文件名称：process.rs
//!
//! 文件功能：
//! macOS 基础进程操作：结束进程、探活、优先级、打开目录。
//!
//! 主要职责：
//! - SIGTERM 优雅关闭，SIGKILL 强制结束
//! - 用 libc kill / signal 0 探活
//! - 读取 nice 值
//! - 使用 open 打开目录
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::path::Path;
use std::process::Command;

use crate::errors::AppError;
use crate::models::KillMode;

/// 按指定信号结束当前 PID，不杀父进程。
pub fn kill_process(pid: u32, mode: KillMode) -> Result<(), AppError> {
    // SIGTERM 让进程自己收尾；SIGKILL 才能保证内核立刻回收监听端口。
    let signal = match mode {
        KillMode::Graceful => libc::SIGTERM,
        KillMode::Force => libc::SIGKILL,
    };
    let result = unsafe { libc::kill(pid as i32, signal) };
    if result == 0 || !process_is_running(pid) {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EPERM) {
            Err(AppError::PermissionDenied)
        } else if error.raw_os_error() == Some(libc::ESRCH) {
            Err(AppError::ProcessNotFound)
        } else {
            Err(AppError::SystemError(error.to_string()))
        }
    }
}

/// signal 0 只探活，不发送实际信号。
pub(crate) fn process_is_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// 读取 nice 值并映射为前端可翻译的优先级码。
pub fn process_priority(pid: u32) -> String {
    let nice = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid as u32) };
    if nice < 0 {
        "high".to_string()
    } else if nice > 0 {
        "idle".to_string()
    } else {
        "normal".to_string()
    }
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

    Command::new("open")
        .arg(dir.as_os_str())
        .spawn()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    Ok(())
}

/// 发出结束信号后立即返回，供面板反复拉起时快速再杀。
pub fn request_kill(pid: u32, mode: KillMode) {
    let _ = kill_process(pid, mode);
}
