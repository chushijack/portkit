//! 文件名称：process.rs
//!
//! 文件功能：
//! 使用 sysinfo 查询进程快照，供服务层组装详情。
//!
//! 主要职责：
//! - 显式刷新命令行与用户，sysinfo 默认刷新不含这两项
//! - 读取名称、路径、CPU 与内存；启动时间为 0 时用系统 API 补一次
//! - 收集同名祖先 PID，避免只杀监听子进程
//! - 按控制者子树找出同名进程，并把父进程分成 Application / Supervisor
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

use crate::errors::AppError;
use crate::models::ProcessIdentity;

/// 进程查询快照。CPU 需要两次刷新后才有意义。
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    #[allow(dead_code)]
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub command: String,
    pub user: String,
    pub start_time: u64,
    pub occupied_seconds: u64,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub running: bool,
}

/// 查询指定 PID 的进程信息。
///
/// `include_cpu` 为 true 时会等待最小采样间隔，避免 CPU 一直为 0。
pub fn query_process(pid: u32, include_cpu: bool) -> Result<ProcessSnapshot, AppError> {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[sys_pid]),
        true,
        process_refresh_kind(),
    );

    if include_cpu {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[sys_pid]),
            true,
            process_refresh_kind(),
        );
    }

    let process = system
        .process(sys_pid)
        .ok_or(AppError::ProcessNotFound)?;

    let start_time = resolve_start_time(pid, process.start_time());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(start_time);
    let occupied_seconds = uptime_seconds(start_time, now);

    let users = Users::new_with_refreshed_list();
    let raw_name = process.name().to_string_lossy().to_string();
    let name = strip_exe_suffix(&raw_name);
    let path = process
        .exe()
        .map(|exe| exe.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(ProcessSnapshot {
        pid,
        name,
        path,
        command: command_line(process),
        user: resolve_user(process, pid, &users),
        start_time,
        occupied_seconds,
        cpu_usage: process.cpu_usage(),
        memory_bytes: process.memory(),
        running: true,
    })
}

/// 读取进程实例身份，供控制者分析和端口释放后的比对。
pub fn process_identity(pid: u32) -> Result<ProcessIdentity, AppError> {
    let snapshot = query_process(pid, false)?;
    let command_line = if snapshot.command.is_empty() {
        None
    } else {
        Some(snapshot.command)
    };
    Ok(ProcessIdentity {
        pid,
        start_time: snapshot.start_time,
        executable: snapshot.path,
        command_line,
        name: snapshot.name,
    })
}

/// 向上收集与当前进程同名的祖先 PID。
///
/// mysqld 常有「父 mysqld + 子 mysqld」，只杀监听的子进程时父进程会立刻再 fork。
/// 返回顺序：离当前进程最近的同名父进程在前。遇到不同名进程即停止，不会包含面板主进程。
pub fn same_named_ancestors(pid: u32, process_name: &str) -> Vec<u32> {
    let self_name = process_name.trim().to_lowercase();
    if self_name.is_empty() || self_name == "-" {
        return Vec::new();
    }
    let mut ancestors = Vec::new();
    let mut current = pid;
    let mut seen = HashSet::new();
    for _ in 0..8 {
        if !seen.insert(current) {
            break;
        }
        let Some(parent_pid) = process_parent(current) else {
            break;
        };
        if parent_pid == 0 || parent_pid == current {
            break;
        }
        let Ok(parent) = query_process(parent_pid, false) else {
            break;
        };
        if parent.name.to_lowercase() != self_name {
            break;
        }
        ancestors.push(parent_pid);
        current = parent_pid;
    }
    ancestors
}

/// 同名包装进程（如 mysqld 下面还有 mysqld）视为 Supervisor。
///
/// 终端里 pnpm → node → node 会挂多个同名进程，那是普通进程树，不是守护程序。
pub fn looks_like_supervisor(worker_pid: u32, worker_name: &str) -> bool {
    !same_named_ancestors(worker_pid, worker_name).is_empty()
}

/// 把非系统宿主父进程分成 Application 或 Supervisor。
pub fn parent_controller(
    controller_pid: u32,
    controller_name: String,
    worker: &ProcessIdentity,
) -> crate::models::ProcessController {
    if looks_like_supervisor(worker.pid, &worker.name) {
        crate::models::ProcessController::Supervisor {
            pid: controller_pid,
            name: controller_name,
        }
    } else {
        crate::models::ProcessController::Application {
            pid: controller_pid,
            name: controller_name,
        }
    }
}

/// 找出某个控制者进程下面、与给定名称相同的所有 PID（不含控制者自己）。
///
/// 面板重新拉起后 PID 会变，不能只看最初那两个 mysqld。
pub fn pids_named_under(root_pid: u32, process_name: &str) -> Vec<u32> {
    let self_name = process_name.trim().to_lowercase();
    if self_name.is_empty() || self_name == "-" || root_pid == 0 {
        return Vec::new();
    }
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        process_refresh_kind(),
    );

    let mut parent_of: HashMap<u32, u32> = HashMap::new();
    let mut named = Vec::new();
    for (pid, process) in system.processes() {
        let current = pid.as_u32();
        if let Some(parent) = process.parent() {
            parent_of.insert(current, parent.as_u32());
        }
        let raw_name = process.name().to_string_lossy().to_string();
        if strip_exe_suffix(&raw_name).to_lowercase() == self_name && current != root_pid {
            named.push(current);
        }
    }
    let mut named: Vec<(u32, usize)> = named
        .into_iter()
        .filter_map(|pid| ancestor_depth(pid, root_pid, &parent_of).map(|depth| (pid, depth)))
        .collect();
    // 离控制者更近的先结束，taskkill /T 才能带上还没退出的子进程。
    named.sort_by_key(|(pid, depth)| (*depth, *pid));
    named.into_iter().map(|(pid, _)| pid).collect()
}

/// 返回 pid 到 root 的层数；pid 就是 root 时为 0。
fn ancestor_depth(pid: u32, root: u32, parent_of: &HashMap<u32, u32>) -> Option<usize> {
    let mut current = pid;
    let mut seen = HashSet::new();
    let mut depth = 0usize;
    for _ in 0..16 {
        if current == root {
            return Some(depth);
        }
        if !seen.insert(current) {
            return None;
        }
        let parent = parent_of.get(&current).copied()?;
        if parent == 0 || parent == current {
            return None;
        }
        depth += 1;
        current = parent;
    }
    None
}

fn process_parent(pid: u32) -> Option<u32> {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[sys_pid]),
        true,
        process_refresh_kind(),
    );
    system
        .process(sys_pid)?
        .parent()
        .map(|parent| parent.as_u32())
}

const SYSTEM_HOSTS: &[&str] = &[
    "explorer",
    "cmd",
    "powershell",
    "pwsh",
    "conhost",
    "windowsterminal",
    "openconsole",
    "wt",
    "services",
    "svchost",
    "wininit",
    "winlogon",
    "csrss",
    "smss",
    "system",
    "idle",
    "launchd",
    "init",
    "systemd",
    "login",
    "bash",
    "zsh",
    "sh",
    "fish",
    "kernel_task",
    "loginwindow",
    "finder",
    "terminal",
    "windowserver",
    "cfprefsd",
];

/// 系统宿主不算生命周期控制者。
pub(crate) fn is_system_host(name: &str) -> bool {
    let normalized = name
        .trim()
        .trim_end_matches(".exe")
        .trim_end_matches(".EXE")
        .to_lowercase();
    SYSTEM_HOSTS.iter().any(|item| *item == normalized)
}

/// Windows 进程名带 `.exe`，列表展示时去掉以贴近原型。
fn strip_exe_suffix(name: &str) -> String {
    name.strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name)
        .to_string()
}

/// 批量查询多个 PID，扫描列表时避免逐个刷新系统。
pub fn query_processes(pids: &[u32]) -> std::collections::HashMap<u32, ProcessSnapshot> {
    let mut unique: Vec<Pid> = pids
        .iter()
        .copied()
        .filter(|pid| *pid > 0)
        .map(Pid::from_u32)
        .collect();
    unique.sort();
    unique.dedup();

    let mut system = System::new();
    if !unique.is_empty() {
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&unique),
            true,
            process_refresh_kind(),
        );
    }

    let users = Users::new_with_refreshed_list();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let mut map = std::collections::HashMap::new();
    for pid in unique {
        let Some(process) = system.process(pid) else {
            continue;
        };
        let start_time = resolve_start_time(pid.as_u32(), process.start_time());
        let raw_name = process.name().to_string_lossy().to_string();
        map.insert(
            pid.as_u32(),
            ProcessSnapshot {
                pid: pid.as_u32(),
                name: strip_exe_suffix(&raw_name),
                path: process
                    .exe()
                    .map(|exe| exe.to_string_lossy().to_string())
                    .unwrap_or_default(),
                command: command_line(process),
                user: resolve_user(process, pid.as_u32(), &users),
                start_time,
                occupied_seconds: uptime_seconds(start_time, now),
                cpu_usage: 0.0,
                memory_bytes: process.memory(),
                running: true,
            },
        );
    }
    map
}

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_memory()
        .with_cpu()
        .with_exe(UpdateKind::Always)
        .with_cmd(UpdateKind::Always)
        .with_user(UpdateKind::Always)
}

fn command_line(process: &Process) -> String {
    process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// sysinfo 对受保护进程常返回 0，0 会被格式化成 1970-01-01。
fn resolve_start_time(pid: u32, reported: u64) -> u64 {
    if reported > 0 {
        return reported;
    }
    fallback_start_time(pid).unwrap_or(0)
}

fn uptime_seconds(start_time: u64, now: u64) -> u64 {
    if start_time == 0 {
        0
    } else {
        now.saturating_sub(start_time)
    }
}

fn resolve_user(process: &Process, pid: u32, users: &Users) -> String {
    if let Some(name) = process
        .user_id()
        .and_then(|user_id| users.get_user_by_id(user_id))
        .map(|item| item.name().to_string())
        .filter(|name| !name.is_empty() && name != "-")
    {
        return name;
    }
    fallback_account_name(pid).unwrap_or_else(|| "-".to_string())
}

#[cfg(windows)]
fn fallback_account_name(pid: u32) -> Option<String> {
    super::windows::process_account_name(pid)
}

#[cfg(not(windows))]
fn fallback_account_name(_pid: u32) -> Option<String> {
    None
}

#[cfg(windows)]
fn fallback_start_time(pid: u32) -> Option<u64> {
    super::windows::process_start_unix(pid)
}

#[cfg(not(windows))]
fn fallback_start_time(_pid: u32) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::ancestor_depth;
    use std::collections::HashMap;

    #[test]
    fn named_group_hangs_under_panel_not_explorer() {
        let mut parent_of = HashMap::new();
        parent_of.insert(26536, 50564);
        parent_of.insert(50564, 36656);
        parent_of.insert(36656, 14936);
        parent_of.insert(14936, 14840);

        assert_eq!(ancestor_depth(26536, 36656, &parent_of), Some(2));
        assert_eq!(ancestor_depth(50564, 36656, &parent_of), Some(1));
        assert_eq!(ancestor_depth(14936, 36656, &parent_of), None);
    }

    #[test]
    fn pids_named_under_skips_blank_names() {
        assert!(super::pids_named_under(1, "").is_empty());
        assert!(super::pids_named_under(1, "-").is_empty());
        assert!(super::pids_named_under(0, "mysqld").is_empty());
    }

    #[test]
    fn blank_names_are_not_supervisors() {
        assert!(!super::looks_like_supervisor(1, ""));
        assert!(!super::looks_like_supervisor(1, "-"));
    }

    #[test]
    fn keeps_reported_start_time() {
        assert_eq!(super::resolve_start_time(1, 1_700_000_000), 1_700_000_000);
    }

    #[test]
    fn unknown_start_has_no_uptime() {
        assert_eq!(super::uptime_seconds(0, 1_700_000_000), 0);
        assert_eq!(super::uptime_seconds(100, 130), 30);
    }
}
