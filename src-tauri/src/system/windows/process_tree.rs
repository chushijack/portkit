//! 文件名称：process_tree.rs
//!
//! 文件功能：
//! 构建 Windows 进程所有权树，从目标 PID 向上找出生命周期控制者。
//!
//! 主要职责：
//! - 向上最多 8 层，并防止 PID 循环
//! - 跳过 explorer / cmd / services 等系统宿主
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use crate::system::process::is_system_host;
use std::collections::HashSet;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// 进程树节点。children 在所有权树里表示被当前节点拉起的下一层。
#[derive(Debug, Clone)]
pub struct ProcessTreeNode {
    pub pid: u32,
    pub name: String,
    pub parent_pid: u32,
    pub children: Vec<ProcessTreeNode>,
}

const MAX_DEPTH: usize = 8;

/// 从目标 PID 向上构建「控制者 → 目标」的所有权树。
///
/// 返回：树根是最上层祖先；找不到目标进程时返回 None。
pub fn ownership_tree(pid: u32) -> Option<ProcessTreeNode> {
    let chain = ancestor_chain(pid)?;
    nest_from_root(chain)
}

/// 把所有权树打成可日志输出的文本，会读 parent_pid 与 children。
pub fn format_tree(node: &ProcessTreeNode) -> String {
    format_tree_at(node, 0)
}

fn format_tree_at(node: &ProcessTreeNode, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let mut out = format!(
        "{indent}{} pid={} parent={}\n",
        node.name, node.pid, node.parent_pid
    );
    for child in &node.children {
        out.push_str(&format_tree_at(child, depth + 1));
    }
    out
}

/// 从父链里找出第一个非系统宿主、且不是自身的控制者。
///
/// 返回：控制者 PID 与进程名。
pub fn supervisor_from_tree(pid: u32, process_name: &str) -> Option<(u32, String)> {
    let chain = ancestor_chain(pid)?;
    let self_name = process_name.trim().to_lowercase();
    chain.into_iter().skip(1).find_map(|node| {
        let name = node.name.to_lowercase();
        if name.is_empty() || name == "-" || name == self_name || is_system_host(&name) {
            return None;
        }
        Some((node.pid, node.name))
    })
}

fn ancestor_chain(pid: u32) -> Option<Vec<ProcessTreeNode>> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    let mut chain = Vec::new();
    let mut current = pid;
    let mut seen = HashSet::new();
    for _ in 0..MAX_DEPTH {
        if !seen.insert(current) {
            break;
        }
        let Some(process) = system.process(Pid::from_u32(current)) else {
            break;
        };
        let raw_name = process.name().to_string_lossy().to_string();
        let parent_pid = process.parent().map(|parent| parent.as_u32()).unwrap_or(0);
        chain.push(ProcessTreeNode {
            pid: current,
            name: strip_exe(&raw_name),
            parent_pid,
            children: Vec::new(),
        });
        if parent_pid == 0 || parent_pid == current {
            break;
        }
        current = parent_pid;
    }
    if chain.is_empty() {
        None
    } else {
        Some(chain)
    }
}

fn nest_from_root(mut chain: Vec<ProcessTreeNode>) -> Option<ProcessTreeNode> {
    // ancestor_chain 是「目标 → 祖先」，反过来才能得到控制者在根、目标在叶子。
    chain.reverse();
    let mut current = chain.pop()?;
    while let Some(mut parent) = chain.pop() {
        parent.children.push(current);
        current = parent;
    }
    Some(current)
}

fn strip_exe(name: &str) -> String {
    name.strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::system::process::is_system_host;

    #[test]
    fn skips_shell_and_scm_hosts() {
        assert!(is_system_host("explorer.exe"));
        assert!(is_system_host("cmd"));
        assert!(is_system_host("WindowsTerminal"));
        assert!(is_system_host("OpenConsole.exe"));
        assert!(!is_system_host("node"));
        assert!(!is_system_host("mysqld"));
    }
}
