//! 文件名称：launchd.rs
//!
//! 文件功能：
//! 按 PID 识别并停止 launchd 管理的 LaunchDaemon / LaunchAgent。
//!
//! 主要职责：
//! - 用 launchctl list / print 把 PID 映射到 Label
//! - 在 LaunchDaemons / LaunchAgents 目录核对 plist
//! - bootout，失败再 launchctl stop
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::AppError;
use crate::models::ProcessIdentity;

/// launchd 任务所在域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchDomain {
    System,
    Gui(u32),
}

/// 与 PID 对应的 launchd 任务。
#[derive(Debug, Clone)]
pub struct LaunchJob {
    pub label: String,
    pub domain: LaunchDomain,
    pub plist_path: Option<PathBuf>,
}

/// 按 PID 查找 launchd 任务。
pub fn find_job_by_pid(pid: u32, identity: Option<&ProcessIdentity>) -> Option<LaunchJob> {
    if let Some(job) = job_from_launchctl_list(pid) {
        return Some(job);
    }
    if let Some(job) = job_from_print_pid(pid) {
        return Some(job);
    }
    identity.and_then(job_from_plist_scan)
}

/// 若该 PID 属于 launchd 任务，则 bootout / stop。
///
/// 不是 launchd 任务时返回 Ok(false)。
pub fn stop_owning_service(pid: u32) -> Result<bool, AppError> {
    let identity = crate::system::process_identity(pid).ok();
    let Some(job) = find_job_by_pid(pid, identity.as_ref()) else {
        return Ok(false);
    };
    if is_protected_apple_job(&job) {
        return Err(AppError::ProtectedProcess);
    }
    stop_job(&job)?;
    Ok(true)
}

/// 停止已识别的 launchd 任务。
pub fn stop_job(job: &LaunchJob) -> Result<(), AppError> {
    let domain = domain_target(job.domain);
    let service_target = format!("{domain}/{}", job.label);
    if run_launchctl(&["bootout", &service_target]).is_ok() {
        return Ok(());
    }
    if let Some(path) = &job.plist_path {
        if run_launchctl(&["bootout", &domain, &path.to_string_lossy()]).is_ok() {
            return Ok(());
        }
    }
    run_launchctl(&["stop", &job.label])?;
    Ok(())
}

fn is_protected_apple_job(job: &LaunchJob) -> bool {
    let apple_label = job.label.starts_with("com.apple.");
    let system_plist = job
        .plist_path
        .as_ref()
        .is_some_and(|path| path.starts_with("/System/"));
    apple_label && system_plist
}

fn domain_target(domain: LaunchDomain) -> String {
    match domain {
        LaunchDomain::System => "system".to_string(),
        LaunchDomain::Gui(uid) => format!("gui/{uid}"),
    }
}

fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

fn job_from_launchctl_list(pid: u32) -> Option<LaunchJob> {
    let output = run_launchctl(&["list"]).ok()?;
    let label = parse_launchctl_list(&output, pid)?;
    Some(job_from_label(label))
}

fn job_from_print_pid(pid: u32) -> Option<LaunchJob> {
    let target = format!("pid/{pid}");
    let output = run_launchctl(&["print", &target]).ok()?;
    if let Some(path) = parse_plist_path(&output) {
        let label = plist_label(&read_plist(&path).unwrap_or_default())
            .or_else(|| parse_print_label(&output))
            .unwrap_or_default();
        if label.is_empty() {
            return None;
        }
        return Some(LaunchJob {
            domain: domain_from_path(&path),
            label,
            plist_path: Some(path),
        });
    }
    parse_print_label(&output).map(job_from_label)
}

fn job_from_plist_scan(identity: &ProcessIdentity) -> Option<LaunchJob> {
    for dir in plist_dirs() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("plist") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if !plist_matches_identity(&content, identity) {
                continue;
            }
            let Some(label) = plist_label(&content) else {
                continue;
            };
            return Some(LaunchJob {
                domain: domain_from_path(&path),
                label,
                plist_path: Some(path),
            });
        }
    }
    None
}

fn job_from_label(label: String) -> LaunchJob {
    let plist_path = find_plist_for_label(&label);
    let domain = plist_path
        .as_ref()
        .map(|path| domain_from_path(path))
        .unwrap_or_else(|| LaunchDomain::Gui(current_uid()));
    LaunchJob {
        label,
        domain,
        plist_path,
    }
}

fn find_plist_for_label(label: &str) -> Option<PathBuf> {
    let file_name = format!("{label}.plist");
    for dir in plist_dirs() {
        let path = dir.join(&file_name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn plist_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Library/LaunchDaemons"),
        PathBuf::from("/Library/LaunchAgents"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Library/LaunchAgents"));
    }
    dirs
}

fn domain_from_path(path: &Path) -> LaunchDomain {
    let text = path.to_string_lossy();
    if text.contains("LaunchDaemons") {
        LaunchDomain::System
    } else {
        LaunchDomain::Gui(current_uid())
    }
}

fn read_plist(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn run_launchctl(args: &[&str]) -> Result<String, AppError> {
    let output = Command::new("launchctl")
        .args(args)
        .output()
        .map_err(|error| AppError::SystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            format!("launchctl {} 失败", args.join(" "))
        } else {
            stderr
        };
        return Err(AppError::SystemError(detail));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 解析 `launchctl list`：PID、Status、Label。
pub fn parse_launchctl_list(output: &str, pid: u32) -> Option<String> {
    let needle = pid.to_string();
    for line in output.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let pid_col = parts.next()?;
        if pid_col != needle {
            continue;
        }
        let _status = parts.next()?;
        let label = parts.collect::<Vec<_>>().join(" ");
        if label.is_empty() {
            return None;
        }
        return Some(label);
    }
    None
}

fn parse_plist_path(output: &str) -> Option<PathBuf> {
    for line in output.lines() {
        let trimmed = line.trim();
        let Some(value) = trimmed.strip_prefix("path = ") else {
            continue;
        };
        if value.ends_with(".plist") {
            return Some(PathBuf::from(value));
        }
    }
    None
}

fn parse_print_label(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("label = ") {
            let label = value.trim().trim_matches('"');
            if !label.is_empty() {
                return Some(label.to_string());
            }
        }
    }
    None
}

fn plist_label(content: &str) -> Option<String> {
    xml_string_after_key(content, "Label")
}

fn plist_matches_identity(content: &str, identity: &ProcessIdentity) -> bool {
    let exe = identity.executable.trim();
    if exe.is_empty() {
        return false;
    }
    if let Some(program) = xml_string_after_key(content, "Program") {
        if path_matches(exe, &program) {
            return true;
        }
    }
    xml_strings_in_array_after_key(content, "ProgramArguments")
        .iter()
        .any(|item| path_matches(exe, item) || identity.name.eq_ignore_ascii_case(item))
}

fn path_matches(executable: &str, program: &str) -> bool {
    let left = executable.trim_end_matches('/').to_lowercase();
    let right = program.trim_end_matches('/').to_lowercase();
    left == right || left.ends_with(&right) || right.ends_with(&left)
}

fn xml_string_after_key(content: &str, key: &str) -> Option<String> {
    let marker = format!("<key>{key}</key>");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let from = rest.find("<string>")? + 8;
    let to = rest[from..].find("</string>")?;
    let value = rest[from..from + to].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn xml_strings_in_array_after_key(content: &str, key: &str) -> Vec<String> {
    let marker = format!("<key>{key}</key>");
    let Some(start) = content.find(&marker) else {
        return Vec::new();
    };
    let rest = &content[start + marker.len()..];
    let Some(array_start) = rest.find("<array>") else {
        return Vec::new();
    };
    let rest = &rest[array_start + 7..];
    let Some(array_end) = rest.find("</array>") else {
        return Vec::new();
    };
    let array = &rest[..array_end];
    let mut values = Vec::new();
    let mut cursor = array;
    while let Some(from_rel) = cursor.find("<string>") {
        let from = from_rel + 8;
        let Some(to) = cursor[from..].find("</string>") else {
            break;
        };
        let value = cursor[from..from + to].trim();
        if !value.is_empty() {
            values.push(value.to_string());
        }
        cursor = &cursor[from + to + 9..];
    }
    values
}

#[cfg(test)]
mod tests {
    use super::{parse_launchctl_list, plist_label, xml_string_after_key};

    #[test]
    fn parses_launchctl_list_pid_row() {
        let output = "PID\tStatus\tLabel\n412\t0\thomebrew.mxcl.mysql\n-\t0\tcom.apple.SafariHistoryServiceAgent\n";
        assert_eq!(
            parse_launchctl_list(output, 412).as_deref(),
            Some("homebrew.mxcl.mysql")
        );
        assert_eq!(parse_launchctl_list(output, 1), None);
    }

    #[test]
    fn reads_label_from_plist_xml() {
        let xml = r#"
            <key>Label</key>
            <string>homebrew.mxcl.mysql</string>
            <key>ProgramArguments</key>
            <array>
                <string>/opt/homebrew/opt/mysql/bin/mysqld_safe</string>
            </array>
        "#;
        assert_eq!(plist_label(xml).as_deref(), Some("homebrew.mxcl.mysql"));
        assert_eq!(
            xml_string_after_key(xml, "Label").as_deref(),
            Some("homebrew.mxcl.mysql")
        );
    }
}
