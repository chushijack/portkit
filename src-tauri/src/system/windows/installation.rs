//! 文件名称：installation.rs
//!
//! 文件功能：
//! 判断 Windows 下 PortKit 是否为便携版运行。
//!
//! 主要职责：
//! - 区分 NSIS/MSI 安装目录与解压即用的单 exe 便携包
//! - 供更新服务选择对应安装方式
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-29

use std::path::Path;

/// 当前进程是否以便携版方式运行。
///
/// 安装包会附带 `uninstall.exe` 与 `resources` 目录；GitHub 发布的 zip 仅含单个 exe。
pub fn is_portable_installation() -> bool {
    let Ok(exe_path) = std::env::current_exe() else {
        return true;
    };
    let Some(install_dir) = exe_path.parent() else {
        return true;
    };

    !is_managed_install_dir(install_dir)
}

fn is_managed_install_dir(install_dir: &Path) -> bool {
    install_dir.join("uninstall.exe").is_file() || install_dir.join("resources").is_dir()
}
