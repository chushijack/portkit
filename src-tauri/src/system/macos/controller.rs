//! 文件名称：controller.rs
//!
//! 文件功能：
//! 分析谁在控制 macOS 进程的生命周期。
//!
//! 主要职责：
//! - 优先识别 Docker
//! - 其次识别 LaunchDaemon / LaunchAgent
//! - 否则沿进程树找父进程守护者
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use crate::models::{is_docker_process, ProcessController, ProcessIdentity};

use super::launchd::{find_job_by_pid, LaunchDomain};
use super::process_tree::{format_tree, ownership_tree, parent_is_launchd, supervisor_from_tree};

/// 判断进程的生命周期控制者。
///
/// 顺序：Docker → launchd → Application / Supervisor → Unknown。
pub fn detect_controller(process: &ProcessIdentity) -> ProcessController {
    if let Some(tree) = ownership_tree(process.pid) {
        eprint!("[ProcessTree]\n{}", format_tree(&tree));
    }
    if is_docker_process(
        &process.name,
        &process.executable,
        process.command_line.as_deref(),
    ) {
        return ProcessController::DockerContainer {
            id: docker_id_hint(process),
        };
    }
    if let Some(job) = find_job_by_pid(process.pid, Some(process)) {
        let label = job.label.clone();
        return match job.domain {
            LaunchDomain::System => ProcessController::LaunchDaemon { label },
            LaunchDomain::Gui(_) => ProcessController::LaunchAgent { label },
        };
    }
    if parent_is_launchd(process.pid) {
        return ProcessController::LaunchDaemon {
            label: String::new(),
        };
    }
    if let Some((pid, name)) = supervisor_from_tree(process.pid, &process.name) {
        if is_docker_process(&name, "", None) {
            return ProcessController::DockerContainer { id: "-".to_string() };
        }
        return crate::system::parent_controller(pid, name, process);
    }
    ProcessController::Unknown
}

fn docker_id_hint(process: &ProcessIdentity) -> String {
    let Some(command) = &process.command_line else {
        return "-".to_string();
    };
    command
        .split_whitespace()
        .find(|part| part.len() >= 12 && part.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .unwrap_or("-")
        .to_string()
}
