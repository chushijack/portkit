//! 文件名称：process_service.rs
//!
//! 文件功能：
//! 提供进程详情、打开目录与释放端口业务。
//!
//! 主要职责：
//! - 组装详情页数据
//! - 按控制者类型执行正确停止动作（停服务 / 结束 worker / 不杀守护程序）
//! - 按优雅关闭或强制结束选择信号，并返回真实生命周期结果
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::time::{Duration, Instant};

use crate::config::PROTECTED_PROCESS_NAMES;
use crate::errors::AppError;
use crate::models::{KillMode, KillOutcome, ProcessController, ProcessDetail, ProcessIdentity};
use crate::services::service_detector;
use crate::system;
use crate::utils::{format_local_datetime, format_uptime};

const WATCHDOG_POLL: Duration = Duration::from_millis(200);
const SERVICE_WATCH: Duration = Duration::from_millis(4000);
const WORKER_KILL: Duration = Duration::from_millis(3000);
const GRACEFUL_WAIT: Duration = Duration::from_millis(8000);
const RESTART_WATCH: Duration = Duration::from_millis(10000);
const APP_WATCH: Duration = Duration::from_millis(3000);
const PORT_QUIET: Duration = Duration::from_millis(500);

enum PortAfterKill {
    Released,
    StillOccupied,
    Rebound(ProcessIdentity),
}

/// 查询进程详情，包含 CPU、内存和该进程占用的端口。
pub fn get_process_detail(pid: u32) -> Result<ProcessDetail, AppError> {
    if pid == 0 {
        return Err(AppError::ProcessNotFound);
    }
    let snapshot = system::query_process(pid, true)?;
    let sockets = system::list_sockets().unwrap_or_default();
    let ports = sockets
        .into_iter()
        .filter(|socket| socket.pid == pid)
        .map(|socket| socket.port)
        .collect::<Vec<_>>();

    Ok(ProcessDetail {
        pid,
        name: snapshot.name.clone(),
        display_name: service_detector::detect_runtime_display_name(
            &snapshot.name,
            &snapshot.command,
        ),
        service_labels: service_detector::detect_service_labels(
            &snapshot.name,
            &ports,
            &snapshot.command,
        ),
        status: if snapshot.running {
            "running".to_string()
        } else {
            "exited".to_string()
        },
        user: snapshot.user,
        start_time: format_local_datetime(snapshot.start_time),
        duration: format_uptime(snapshot.start_time, snapshot.occupied_seconds),
        priority: system::process_priority(pid),
        path: snapshot.path,
        command: if snapshot.command.is_empty() {
            "-".to_string()
        } else {
            snapshot.command
        },
        cpu_usage: snapshot.cpu_usage,
        memory_bytes: snapshot.memory_bytes,
        ports,
    })
}

/// 打开进程可执行文件所在目录。
pub fn open_process_directory(pid: u32) -> Result<(), AppError> {
    let snapshot = system::query_process(pid, false)?;
    if snapshot.path.is_empty() {
        return Err(AppError::SystemError("无法获取进程路径".to_string()));
    }
    system::open_directory(&snapshot.path)
}

/// 按控制者类型释放端口：服务只停服务；Supervisor 只结束当前 worker 并检测恢复，不杀面板。
///
/// 结束进程成功不等于端口生命周期结束。Supervisor 禁止返回 Released。
pub fn kill_process(
    pid: u32,
    port: Option<u16>,
    fallback_name: Option<String>,
    mode: KillMode,
) -> Result<KillOutcome, AppError> {
    ensure_killable(pid)?;
    let identity = system::process_identity(pid).unwrap_or_else(|_| ProcessIdentity {
        pid,
        start_time: 0,
        executable: String::new(),
        command_line: None,
        name: fallback_name
            .as_deref()
            .map(sanitize_name)
            .filter(|name| name != "-")
            .unwrap_or_else(|| "process".to_string()),
    });
    let process_name = sanitize_name(&identity.name);
    let controller = system::detect_controller(&identity);
    log_process(
        &identity,
        &controller,
        action_label(&controller, mode),
        "start",
    );

    match &controller {
        ProcessController::DockerContainer { .. } => {
            log_process(
                &identity,
                &controller,
                "skip kill (docker)",
                "requires external stop",
            );
            return Ok(KillOutcome::requires_external(process_name, &controller));
        }
        ProcessController::WindowsService { .. }
        | ProcessController::LaunchDaemon { .. }
        | ProcessController::LaunchAgent { .. } => {
            // SCM / launchd 把杀 PID 当成崩溃，可能立刻拉起新 PID，所以先停服务。
            if !system::stop_owning_service(pid)? {
                return Err(AppError::SystemError(
                    "未能停止对应的系统服务".to_string(),
                ));
            }
            wait_until_instance_gone(&identity, SERVICE_WATCH);
            if same_instance_alive(&identity) && mode.is_force() {
                system::request_kill(pid, mode);
                wait_until_instance_gone(&identity, WORKER_KILL);
            }
            if same_instance_alive(&identity) {
                log_process(
                    &identity,
                    &controller,
                    action_label(&controller, mode),
                    "still running",
                );
                return Err(AppError::ProcessStillRunning);
            }
        }
        ProcessController::Supervisor { pid: controller_pid, .. } => {
            return handle_supervisor(
                &identity,
                &controller,
                process_name,
                *controller_pid,
                port,
                mode,
            );
        }
        ProcessController::Application { pid: controller_pid, .. } => {
            return handle_application(
                &identity,
                &controller,
                process_name,
                Some(*controller_pid),
                port,
                mode,
            );
        }
        ProcessController::Unknown => {
            return handle_application(&identity, &controller, process_name, None, port, mode);
        }
    }

    let Some(port) = port else {
        log_process(&identity, &controller, action_label(&controller, mode), "released (no port)");
        return Ok(KillOutcome::released(process_name, &controller));
    };

    match wait_for_port_outcome(&identity, port, SERVICE_WATCH) {
        PortAfterKill::Released => {
            log_process(&identity, &controller, action_label(&controller, mode), "Released");
            Ok(KillOutcome::released(process_name, &controller))
        }
        PortAfterKill::StillOccupied => {
            log_process(&identity, &controller, action_label(&controller, mode), "still occupied");
            Err(AppError::ProcessStillRunning)
        }
        PortAfterKill::Rebound(new_identity) => {
            if system::detect_controller(&new_identity).is_managed_service()
                && system::stop_owning_service(new_identity.pid).unwrap_or(false)
                && matches!(
                    wait_for_port_outcome(&new_identity, port, SERVICE_WATCH),
                    PortAfterKill::Released
                )
            {
                log_process(
                    &new_identity,
                    &controller,
                    "stop managed service (restarted instance)",
                    "Released",
                );
                return Ok(KillOutcome::released(process_name, &controller));
            }
            log_process(
                &identity,
                &controller,
                action_label(&controller, mode),
                &format!("restart detected pid={}", new_identity.pid),
            );
            Ok(KillOutcome::restarted(
                process_name,
                &controller,
                new_identity.pid,
            ))
        }
    }
}

/// 结束当前 worker 一次，再观察守护程序是否拉起新 PID。禁止 Released。
fn handle_supervisor(
    identity: &ProcessIdentity,
    controller: &ProcessController,
    process_name: String,
    controller_pid: u32,
    port: Option<u16>,
    mode: KillMode,
) -> Result<KillOutcome, AppError> {
    log_process(
        identity,
        controller,
        action_label(controller, mode),
        &format!("terminate worker pid={}", identity.pid),
    );
    terminate_worker_once(identity.pid, &process_name, Some(controller_pid), mode);
    wait_until_instance_gone(identity, worker_wait(mode));
    if same_instance_alive(identity) {
        log_process(identity, controller, action_label(controller, mode), "still running");
        return Err(AppError::ProcessStillRunning);
    }

    match supervisor_watch_result(watch_restart(
        identity,
        controller_pid,
        &process_name,
        port,
        RESTART_WATCH,
    )) {
        SupervisorWatchEnd::Restarted(pid) => {
            log_process(
                identity,
                controller,
                action_label(controller, mode),
                &format!("restart detected pid={pid}"),
            );
            Ok(KillOutcome::restarted(process_name, controller, pid))
        }
        SupervisorWatchEnd::RequireControllerStop => {
            log_process(
                identity,
                controller,
                action_label(controller, mode),
                "require controller stop",
            );
            Ok(KillOutcome::require_controller_stop(process_name, controller))
        }
    }
}

/// 普通进程：结束 worker，短观察窗口内没有同名新 PID 才算 Released。
fn handle_application(
    identity: &ProcessIdentity,
    controller: &ProcessController,
    process_name: String,
    controller_pid: Option<u32>,
    port: Option<u16>,
    mode: KillMode,
) -> Result<KillOutcome, AppError> {
    terminate_worker_once(identity.pid, &process_name, controller_pid, mode);
    wait_until_instance_gone(identity, worker_wait(mode));
    if same_instance_alive(identity) {
        log_process(identity, controller, action_label(controller, mode), "still running");
        return Err(AppError::ProcessStillRunning);
    }

    if let Some(parent) = controller_pid {
        if let Some(pid) = watch_restart(identity, parent, &process_name, port, APP_WATCH) {
            let supervisor = controller.as_supervisor();
            log_process(
                identity,
                &supervisor,
                action_label(&supervisor, mode),
                &format!("restart detected pid={pid}"),
            );
            return Ok(KillOutcome::restarted(process_name, &supervisor, pid));
        }
    } else if let Some(port) = port {
        if let PortAfterKill::Rebound(new_identity) =
            wait_for_port_outcome(identity, port, APP_WATCH)
        {
            log_process(
                identity,
                controller,
                action_label(controller, mode),
                &format!("restart detected pid={}", new_identity.pid),
            );
            return Ok(KillOutcome::restarted(
                process_name,
                controller,
                new_identity.pid,
            ));
        }
    }

    if let Some(port) = port {
        match wait_for_port_outcome(identity, port, Duration::from_millis(0)) {
            PortAfterKill::StillOccupied => {
                log_process(identity, controller, action_label(controller, mode), "still occupied");
                return Err(AppError::ProcessStillRunning);
            }
            PortAfterKill::Rebound(new_identity) => {
                log_process(
                    identity,
                    controller,
                    action_label(controller, mode),
                    &format!("restart detected pid={}", new_identity.pid),
                );
                return Ok(KillOutcome::restarted(
                    process_name,
                    controller,
                    new_identity.pid,
                ));
            }
            PortAfterKill::Released => {}
        }
    }

    log_process(identity, controller, action_label(controller, mode), "Released");
    Ok(KillOutcome::released(process_name, controller))
}

/// 只结束当前这一组 worker，发现新 PID 也不再杀。
fn terminate_worker_once(pid: u32, process_name: &str, skip: Option<u32>, mode: KillMode) {
    let mut targets = Vec::new();
    if let Some(controller_pid) = skip {
        targets = system::pids_named_under(controller_pid, process_name);
    }
    if targets.is_empty() {
        targets.push(pid);
        targets.extend(
            system::same_named_ancestors(pid, process_name)
                .into_iter()
                .filter(|item| skip != Some(*item)),
        );
        targets.retain(|item| *item != 0 && skip != Some(*item));
        targets.reverse();
    }
    for target in targets {
        if ensure_killable(target).is_err() {
            continue;
        }
        system::request_kill(target, mode);
    }
}

fn worker_wait(mode: KillMode) -> Duration {
    match mode {
        KillMode::Graceful => GRACEFUL_WAIT,
        KillMode::Force => WORKER_KILL,
    }
}

/// 同名 exe、同一父进程、新 PID，或端口上出现新实例，视为守护程序恢复 worker。
fn watch_restart(
    original: &ProcessIdentity,
    controller_pid: u32,
    process_name: &str,
    port: Option<u16>,
    timeout: Duration,
) -> Option<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(pid) = restarted_worker(original, controller_pid, process_name, port) {
            return Some(pid);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(WATCHDOG_POLL);
    }
}

fn restarted_worker(
    original: &ProcessIdentity,
    controller_pid: u32,
    process_name: &str,
    port: Option<u16>,
) -> Option<u32> {
    for pid in system::pids_named_under(controller_pid, process_name) {
        let Ok(current) = system::process_identity(pid) else {
            continue;
        };
        if !original.is_same_instance(&current) {
            return Some(pid);
        }
    }
    let listener = port.and_then(listener_identity)?;
    if original.is_same_instance(&listener) {
        return None;
    }
    Some(listener.pid)
}

/// 窗口内是否观察到新 worker。Supervisor 没有新 PID 时不能报 Released。
fn supervisor_watch_result(newcomer: Option<u32>) -> SupervisorWatchEnd {
    match newcomer {
        Some(pid) => SupervisorWatchEnd::Restarted(pid),
        None => SupervisorWatchEnd::RequireControllerStop,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SupervisorWatchEnd {
    Restarted(u32),
    RequireControllerStop,
}

fn action_label(controller: &ProcessController, mode: KillMode) -> String {
    let kind = match controller {
        ProcessController::WindowsService { .. } => "SERVICE_CONTROL_STOP",
        ProcessController::LaunchDaemon { .. } | ProcessController::LaunchAgent { .. } => "bootout",
        ProcessController::Supervisor { .. } => "terminate worker",
        ProcessController::Application { .. } => "Terminate process",
        ProcessController::DockerContainer { .. } => "skip kill (docker)",
        ProcessController::Unknown => "Terminate process",
    };
    format!("{kind} ({})", mode.log_label())
}

fn log_process(
    identity: &ProcessIdentity,
    controller: &ProcessController,
    action: impl AsRef<str>,
    result: impl AsRef<str>,
) {
    let extra = match controller {
        ProcessController::Application { name, pid }
        | ProcessController::Supervisor { name, pid } => format!("{name} ({pid})"),
        ProcessController::WindowsService { name } => format!("service:{name}"),
        ProcessController::LaunchDaemon { label } | ProcessController::LaunchAgent { label } => {
            format!("label:{label}")
        }
        ProcessController::DockerContainer { id } => format!("container:{id}"),
        ProcessController::Unknown => "-".to_string(),
    };
    eprintln!(
        "[Process]\nPID: {}\nName: {}\nController: {}\nLabel: {}\nAction: {}\nResult: {}",
        identity.pid,
        identity.name,
        controller.kind_label(),
        extra,
        action.as_ref(),
        result.as_ref()
    );
}

fn wait_for_port_outcome(
    original: &ProcessIdentity,
    port: u16,
    timeout: Duration,
) -> PortAfterKill {
    let deadline = Instant::now() + timeout;
    let mut empty_since: Option<Instant> = None;
    loop {
        match listener_identity(port) {
            Some(current) if !original.is_same_instance(&current) => {
                return PortAfterKill::Rebound(current);
            }
            Some(_) => {
                empty_since = None;
            }
            None => {
                let started = *empty_since.get_or_insert_with(Instant::now);
                if timeout.is_zero() || started.elapsed() >= PORT_QUIET {
                    return PortAfterKill::Released;
                }
            }
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(WATCHDOG_POLL);
    }
    match listener_identity(port) {
        Some(current) if original.is_same_instance(&current) => {
            if same_instance_alive(original) {
                PortAfterKill::StillOccupied
            } else {
                PortAfterKill::Released
            }
        }
        Some(current) => PortAfterKill::Rebound(current),
        None => PortAfterKill::Released,
    }
}

fn listener_identity(port: u16) -> Option<ProcessIdentity> {
    let pid = system::list_sockets()
        .ok()?
        .into_iter()
        .find(|socket| socket.port == port && socket.pid != 0)
        .map(|socket| socket.pid)?;
    system::process_identity(pid).ok()
}

fn same_instance_alive(original: &ProcessIdentity) -> bool {
    system::process_identity(original.pid)
        .map(|current| original.is_same_instance(&current))
        .unwrap_or(false)
}

fn wait_until_instance_gone(original: &ProcessIdentity, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while same_instance_alive(original) && Instant::now() < deadline {
        std::thread::sleep(WATCHDOG_POLL);
    }
}

fn sanitize_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    trimmed.replace('|', "/")
}

fn ensure_killable(pid: u32) -> Result<(), AppError> {
    if pid == 0 || pid == 4 {
        return Err(AppError::ProtectedProcess);
    }
    if pid == std::process::id() {
        return Err(AppError::SelfProcess);
    }
    let snapshot = system::query_process(pid, false)?;
    if is_protected(&snapshot.name) {
        return Err(AppError::ProtectedProcess);
    }
    Ok(())
}

fn is_protected(process_name: &str) -> bool {
    let normalized = process_name.trim().to_lowercase();
    PROTECTED_PROCESS_NAMES
        .iter()
        .any(|item| *item == normalized)
}

#[cfg(test)]
mod tests {
    use super::{is_protected, supervisor_watch_result, SupervisorWatchEnd};
    use crate::models::ProcessIdentity;
    use crate::system::process::is_system_host;

    #[test]
    fn blocks_windows_critical_process() {
        assert!(is_protected("svchost"));
        assert!(is_protected("csrss"));
        assert!(!is_protected("node"));
    }

    #[test]
    fn identity_treats_reused_pid_as_different_instance() {
        let first = ProcessIdentity {
            pid: 1000,
            start_time: 10,
            executable: "C:\\mysql\\mysqld.exe".into(),
            command_line: None,
            name: "mysqld".into(),
        };
        let reused = ProcessIdentity {
            pid: 1000,
            start_time: 99,
            executable: "C:\\mysql\\mysqld.exe".into(),
            command_line: None,
            name: "mysqld".into(),
        };
        assert!(!first.is_same_instance(&reused));
        assert!(first.is_same_instance(&first));
    }

    #[test]
    fn system_hosts_are_not_supervisors() {
        assert!(is_system_host("explorer.exe"));
        assert!(is_system_host("cmd"));
        assert!(is_system_host("WindowsTerminal"));
        assert!(!is_system_host("node"));
    }

    #[test]
    fn same_named_ancestors_skip_blank_names() {
        assert!(crate::system::same_named_ancestors(1, "").is_empty());
        assert!(crate::system::same_named_ancestors(1, "-").is_empty());
    }

    #[test]
    fn supervisor_never_releases_after_watch() {
        assert_eq!(
            supervisor_watch_result(Some(12940)),
            SupervisorWatchEnd::Restarted(12940)
        );
        assert_eq!(
            supervisor_watch_result(None),
            SupervisorWatchEnd::RequireControllerStop
        );
    }
}
