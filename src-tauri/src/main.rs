//! 文件名称：main.rs
//!
//! 文件功能：
//! PortKit 桌面应用入口，启动 Tauri 运行时。
//!
//! 主要职责：
//! - 关闭 Windows release 控制台窗口
//! - 调用库 crate 启动应用
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    portkit_lib::run();
}
