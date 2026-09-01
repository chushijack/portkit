//! 文件名称：kill.rs
//!
//! 文件功能：
//! 定义释放端口时的进程身份、控制者与结果。
//!
//! 主要职责：
//! - 用 PID + 启动时间 + 可执行文件识别同一进程实例
//! - 跨平台区分 SCM / launchd / Application / Supervisor / Docker
//! - 区分优雅关闭与强制结束
//! - 把释放成功、被重新拉起、需到控制者处停止作为业务结果返回
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use serde::{Deserialize, Serialize};

/// 结束进程的方式。缺省强制结束，与旧调用行为一致。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum KillMode {
    /// 先请进程自行退出：Windows 发 WM_CLOSE，macOS 发 SIGTERM。
    Graceful,
    /// 立即结束：Windows TerminateProcess / taskkill /F，macOS SIGKILL。
    #[default]
    Force,
}

impl KillMode {
    /// 日志用的结束方式。
    pub fn log_label(self) -> &'static str {
        match self {
            KillMode::Graceful => "graceful",
            KillMode::Force => "force",
        }
    }

    /// 是否为强制结束。
    pub fn is_force(self) -> bool {
        matches!(self, KillMode::Force)
    }
}

/// 进程实例身份。PID 会复用，不能只靠 PID 判断是不是同一个进程。
#[derive(Debug, Clone)]
pub struct ProcessIdentity {
    pub pid: u32,
    /// 进程启动时间（Unix 秒），与 PID、可执行文件一起做实例比对。
    pub start_time: u64,
    pub executable: String,
    pub command_line: Option<String>,
    pub name: String,
}

impl ProcessIdentity {
    /// 是否为同一次运行的同一个进程。
    pub fn is_same_instance(&self, other: &Self) -> bool {
        self.pid == other.pid
            && self.start_time == other.start_time
            && executable_key(&self.executable, &self.name) == executable_key(&other.executable, &other.name)
    }
}

fn executable_key(path: &str, name: &str) -> String {
    let raw = if path.trim().is_empty() { name } else { path };
    raw.trim().to_lowercase()
}

/// 谁在控制该进程的生命周期。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessController {
    #[cfg_attr(not(windows), allow(dead_code))]
    WindowsService { name: String },
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    LaunchDaemon { label: String },
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    LaunchAgent { label: String },
    /// 普通父子关系：结束子进程后父进程不会把同名 worker 拉起来。
    Application { pid: u32, name: String },
    /// 守护程序：父进程持续存在，会反复拉起同名 worker（新 PID）。
    Supervisor { pid: u32, name: String },
    DockerContainer { id: String },
    Unknown,
}

impl ProcessController {
    /// 给用户看的控制者名称；未知时返回 `-`，由前端译成「守护程序」。
    pub fn display_name(&self) -> String {
        match self {
            ProcessController::WindowsService { name } => name.clone(),
            ProcessController::LaunchDaemon { label } | ProcessController::LaunchAgent { label } => {
                if label.trim().is_empty() {
                    "launchd".to_string()
                } else {
                    label.clone()
                }
            }
            ProcessController::Application { name, .. }
            | ProcessController::Supervisor { name, .. } => name.clone(),
            ProcessController::DockerContainer { id } => {
                if id.trim().is_empty() || id == "-" {
                    "Docker".to_string()
                } else {
                    format!("Docker {id}")
                }
            }
            ProcessController::Unknown => "-".to_string(),
        }
    }

    /// 日志用的控制者类型。
    pub fn kind_label(&self) -> &'static str {
        match self {
            ProcessController::WindowsService { .. } => "WindowsService",
            ProcessController::LaunchDaemon { .. } => "LaunchDaemon",
            ProcessController::LaunchAgent { .. } => "LaunchAgent",
            ProcessController::Application { .. } => "Application",
            ProcessController::Supervisor { .. } => "Supervisor",
            ProcessController::DockerContainer { .. } => "DockerContainer",
            ProcessController::Unknown => "Unknown",
        }
    }

    /// 传给前端的控制者类型码，用于选择提示文案。
    pub fn kind_code(&self) -> &'static str {
        match self {
            ProcessController::WindowsService { .. } => "windowsService",
            ProcessController::LaunchDaemon { .. } => "launchDaemon",
            ProcessController::LaunchAgent { .. } => "launchAgent",
            ProcessController::Application { .. } => "application",
            ProcessController::Supervisor { .. } => "supervisor",
            ProcessController::DockerContainer { .. } => "dockerContainer",
            ProcessController::Unknown => "unknown",
        }
    }

    /// 是否由系统服务管理器托管（SCM / launchd），应停服务而不是杀 PID。
    pub fn is_managed_service(&self) -> bool {
        matches!(
            self,
            ProcessController::WindowsService { .. }
                | ProcessController::LaunchDaemon { .. }
                | ProcessController::LaunchAgent { .. }
        )
    }

    /// 观察到来自同一父进程的重新拉起后，把 Application 升级为 Supervisor。
    pub fn as_supervisor(&self) -> Self {
        match self {
            ProcessController::Application { pid, name } => ProcessController::Supervisor {
                pid: *pid,
                name: name.clone(),
            },
            other => other.clone(),
        }
    }
}

/// 名称、路径或命令行是否像 Docker 相关进程。
pub fn is_docker_process(name: &str, executable: &str, command: Option<&str>) -> bool {
    let blob = format!(
        "{} {} {}",
        name,
        executable,
        command.unwrap_or("")
    )
    .to_lowercase();
    const MARKERS: &[&str] = &[
        "com.docker",
        "docker.app",
        "docker desktop",
        "docker-proxy",
        "vpnkit",
        "docker-backend",
        "com.docker.backend",
        "com.docker.hyperkit",
        "com.apple.virtualization.virtualmachine",
    ];
    if MARKERS.iter().any(|marker| blob.contains(marker)) {
        return true;
    }
    let name_only = name.trim().to_lowercase();
    name_only == "docker" || name_only == "dockerd" || name_only.starts_with("docker-")
}

/// 结束进程并观察端口后的业务结果。被重新拉起或需到外部停止不是 API 失败。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum KillOutcome {
    Released {
        process: String,
        controller: String,
        controller_kind: String,
    },
    ProcessRestarted {
        process: String,
        controller: String,
        controller_kind: String,
        pid: u32,
    },
    RequiresExternalStop {
        process: String,
        controller: String,
        controller_kind: String,
    },
    /// 进程由守护程序管理；当前 worker 已结束，需用户在对应软件里停服务。
    RequireControllerStop {
        process: String,
        controller: String,
        controller_kind: String,
    },
}

impl KillOutcome {
    /// 释放成功时带上已分析出的控制者。
    pub fn released(process: String, controller: &ProcessController) -> Self {
        KillOutcome::Released {
            process,
            controller: controller.display_name(),
            controller_kind: controller.kind_code().to_string(),
        }
    }

    /// 端口被新实例占用时带回控制者和新 PID。
    pub fn restarted(process: String, controller: &ProcessController, pid: u32) -> Self {
        KillOutcome::ProcessRestarted {
            process,
            controller: controller.display_name(),
            controller_kind: controller.kind_code().to_string(),
            pid,
        }
    }

    /// 进程由 Docker 等外部运行时管理，PortKit 不直接结束。
    pub fn requires_external(process: String, controller: &ProcessController) -> Self {
        KillOutcome::RequiresExternalStop {
            process,
            controller: controller.display_name(),
            controller_kind: controller.kind_code().to_string(),
        }
    }

    /// 守护程序仍在管理该进程，PortKit 不结束控制者本身。
    pub fn require_controller_stop(process: String, controller: &ProcessController) -> Self {
        KillOutcome::RequireControllerStop {
            process,
            controller: controller.display_name(),
            controller_kind: controller.kind_code().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_docker_process, KillMode, ProcessController};

    #[test]
    fn kill_mode_defaults_to_force() {
        assert_eq!(KillMode::default(), KillMode::Force);
        let graceful: KillMode = serde_json::from_str("\"graceful\"").expect("graceful");
        let force: KillMode = serde_json::from_str("\"force\"").expect("force");
        assert_eq!(graceful, KillMode::Graceful);
        assert_eq!(force, KillMode::Force);
    }

    #[test]
    fn detects_docker_desktop_backend() {
        assert!(is_docker_process(
            "com.docker.backend",
            "/Applications/Docker.app/Contents/MacOS/com.docker.backend",
            None
        ));
        assert!(is_docker_process("docker-proxy", "", Some("--container-id abc")));
        assert!(!is_docker_process("mysqld", "/opt/homebrew/opt/mysql/bin/mysqld", None));
    }

    #[test]
    fn application_upgrades_to_supervisor_after_restart() {
        let app = ProcessController::Application {
            pid: 36656,
            name: "panel".into(),
        };
        match app.as_supervisor() {
            ProcessController::Supervisor { pid, name } => {
                assert_eq!(pid, 36656);
                assert_eq!(name, "panel");
            }
            other => panic!("expected supervisor, got {other:?}"),
        }
    }
}
