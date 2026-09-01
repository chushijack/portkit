//! 文件名称：controller.rs
//!
//! 文件功能：
//! 分析谁在控制 Windows 进程的生命周期。
//!
//! 主要职责：
//! - 优先识别 Windows 服务
//! - 否则沿进程树把父进程分成 Application 或 Supervisor
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use crate::models::{is_docker_process, ProcessController, ProcessIdentity};

use super::process_tree::{format_tree, ownership_tree, supervisor_from_tree};
use super::service::service_name_by_pid;

/// 判断进程的生命周期控制者。
///
/// 顺序：Docker → Windows 服务 → Application / Supervisor → Unknown。
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
            id: "-".to_string(),
        };
    }
    if let Some(name) = service_name_by_pid(process.pid) {
        if !name.trim().is_empty() {
            return ProcessController::WindowsService { name };
        }
    }
    if let Some((pid, name)) = supervisor_from_tree(process.pid, &process.name) {
        return crate::system::parent_controller(pid, name, process);
    }
    ProcessController::Unknown
}
