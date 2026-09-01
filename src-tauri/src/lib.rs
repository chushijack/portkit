//! 文件名称：lib.rs
//!
//! 文件功能：
//! PortKit Tauri 应用装配入口。
//!
//! 主要职责：
//! - 注册插件、命令和共享状态
//! - 创建系统托盘
//! - 注册全局快捷键
//! - 处理关闭到托盘
//! - 快速搜索窗口失焦后自动隐藏
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

mod commands;
mod config;
mod errors;
mod models;
mod repositories;
mod services;
mod state;
mod system;
mod utils;

use std::sync::Mutex;

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::commands::window::{
    emit_tray_scan, show_main_window, toggle_quick_search_window,
};
use crate::services::settings_service;
use crate::state::AppState;
use crate::utils::{resolve_ui_locale, UiLocale};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;
            let settings = settings_service::load(&data_dir).unwrap_or_default();
            app.manage(AppState {
                data_dir,
                background_run: Mutex::new(settings.background_run),
            });

            build_tray(app.handle(), resolve_ui_locale(&settings.locale))?;
            register_global_shortcut(app.handle());
            bind_close_to_tray(app.handle());
            bind_quick_search_auto_hide(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::port::scan_ports,
            commands::port::get_common_ports,
            commands::process::get_process_detail,
            commands::process::open_process_directory,
            commands::process::kill_process,
            commands::favorite::list_favorites,
            commands::favorite::add_favorite,
            commands::favorite::update_favorite_note,
            commands::favorite::remove_favorite,
            commands::history::list_history,
            commands::history::clear_history,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::release::get_releases,
            commands::release::get_resolved_releases,
            commands::update::get_app_version,
            commands::update::check_update,
            commands::update::download_and_install_update,
            commands::update::restart_app,
            commands::window::hide_quick_search,
            commands::window::open_main_route,
        ])
        .run(tauri::generate_context!())
        .expect("启动 PortKit 失败");
}

/// 按当前语言重建托盘菜单，保存设置后会再调用一次。
pub(crate) fn rebuild_tray(app: &tauri::AppHandle, locale: UiLocale) -> tauri::Result<()> {
    let menu = build_tray_menu(app, locale)?;
    if let Some(tray) = app.tray_by_id("portkit-tray") {
        tray.set_menu(Some(menu))?;
        return Ok(());
    }
    build_tray(app, locale)
}

fn tray_label(locale: UiLocale, zh: &'static str, en: &'static str, ja: &'static str) -> &'static str {
    match locale {
        UiLocale::ZhCn => zh,
        UiLocale::En => en,
        UiLocale::Ja => ja,
    }
}

fn build_tray_menu(app: &tauri::AppHandle, locale: UiLocale) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let scan = MenuItemBuilder::with_id(
        "scan",
        tray_label(locale, "扫描端口", "Scan ports", "ポートをスキャン"),
    )
    .accelerator("CmdOrCtrl+R")
    .build(app)?;
    let detail = MenuItemBuilder::with_id(
        "detail",
        tray_label(locale, "查看详情", "View details", "詳細を見る"),
    )
    .accelerator("CmdOrCtrl+P")
    .build(app)?;
    let release = MenuItemBuilder::with_id(
        "release",
        tray_label(locale, "释放端口", "Release port", "ポートを解放"),
    )
    .accelerator("CmdOrCtrl+K")
    .build(app)?;
    let open_main = MenuItemBuilder::with_id(
        "open-main",
        tray_label(locale, "打开主窗口", "Open main window", "メインウィンドウを開く"),
    )
    .build(app)?;
    let settings = MenuItemBuilder::with_id(
        "settings",
        tray_label(locale, "设置", "Settings", "設定"),
    )
    .build(app)?;
    let quit = MenuItemBuilder::with_id(
        "quit",
        tray_label(locale, "退出 PortKit", "Quit PortKit", "PortKit を終了"),
    )
    .accelerator("CmdOrCtrl+Q")
    .build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    MenuBuilder::new(app)
        .items(&[
            &scan, &detail, &release, &separator, &open_main, &settings, &separator, &quit,
        ])
        .build()
}

fn build_tray(app: &tauri::AppHandle, locale: UiLocale) -> tauri::Result<()> {
    let menu = build_tray_menu(app, locale)?;

    let mut builder = TrayIconBuilder::with_id("portkit-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("PortKit")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "scan" => {
                let _ = emit_tray_scan(app);
            }
            "detail" | "open-main" => {
                let _ = show_main_window(app);
            }
            "release" => {
                let _ = toggle_quick_search_window(app);
            }
            "settings" => {
                let _ = commands::window::open_main_route(app.clone(), "/settings".to_string());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn register_global_shortcut(app: &tauri::AppHandle) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let shortcut = "CommandOrControl+Shift+P";
    if let Err(error) = app.global_shortcut().on_shortcut(shortcut, {
        let handle = app.clone();
        move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = toggle_quick_search_window(&handle);
            }
        }
    }) {
        eprintln!("注册全局快捷键失败：{error}");
    }
}

fn bind_close_to_tray(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let handle = app.clone();
        let _ = window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let background = handle
                    .try_state::<AppState>()
                    .and_then(|state| state.background_run.lock().ok().map(|flag| *flag))
                    .unwrap_or(true);
                if background {
                    api.prevent_close();
                    if let Some(main) = handle.get_webview_window("main") {
                        let _ = main.hide();
                    }
                }
            }
        });
    }
}

/// 切到其他应用后置顶搜索窗还会浮着，Esc 也进不了网页，失焦就关掉。
fn bind_quick_search_auto_hide(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("quick-search") else {
        return;
    };
    let focused_at = std::sync::Mutex::new(std::time::Instant::now());
    let handle = app.clone();
    let _ = window.on_window_event(move |event| match event {
        WindowEvent::Focused(true) => {
            if let Ok(mut time) = focused_at.lock() {
                *time = std::time::Instant::now();
            }
        }
        WindowEvent::Focused(false) => {
            // show + set_focus 时 Windows 会紧跟着丢一次焦点，不能立刻藏。
            let ignore = focused_at
                .lock()
                .ok()
                .map(|time| time.elapsed() < std::time::Duration::from_millis(200))
                .unwrap_or(false);
            if ignore {
                return;
            }
            if let Some(quick) = handle.get_webview_window("quick-search") {
                let _ = quick.hide();
            }
        }
        _ => {}
    });
}
