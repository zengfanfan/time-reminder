mod reminder;

use reminder::ReminderManager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;

// ── App config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub autostart: bool,
    pub quit_on_close: bool,
    pub minimize_to_tray: bool,
    pub locale: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            quit_on_close: false,
            minimize_to_tray: false,
            locale: "en".to_string(),
        }
    }
}

fn config_path() -> std::path::PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("time-veil");
    std::fs::create_dir_all(&p).ok();
    p.push("app-config.json");
    p
}

impl AppConfig {
    pub fn load() -> Self {
        let p = config_path();
        if p.exists() {
            if let Ok(data) = std::fs::read_to_string(&p) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), data);
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

pub struct AppState {
    pub reminder_manager: Mutex<ReminderManager>,
    pub app_config: Mutex<AppConfig>,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn get_reminders(state: tauri::State<'_, AppState>) -> Vec<reminder::ReminderConfig> {
    state.reminder_manager.lock().unwrap().get_all()
}

#[tauri::command]
fn get_countdowns() -> Vec<serde_json::Value> {
    if let Ok(lock) = reminder::LAST_COUNTDOWNS.lock() {
        if let Some(ref snapshot) = *lock {
            return snapshot
                .iter()
                .map(|(id, r)| serde_json::json!({ "id": id, "remaining": r }))
                .collect();
        }
    }
    vec![]
}

#[tauri::command]
fn save_reminder(
    state: tauri::State<'_, AppState>,
    config: reminder::ReminderConfig,
) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    let id = config.id.clone();
    let (_, needs_reset) = manager.upsert_checked(config);
    if needs_reset {
        manager.save(&id).map_err(|e| e.to_string())
    } else {
        manager.save_silent().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn delete_reminder(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    manager.remove(&id);
    manager.save_silent().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_reminder(
    state: tauri::State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    manager.set_enabled(&id, enabled);
    if enabled {
        manager.save(&id).map_err(|e| e.to_string())
    } else {
        manager.save_silent().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_app_config(state: tauri::State<'_, AppState>) -> AppConfig {
    state.app_config.lock().unwrap().clone()
}

#[tauri::command]
fn set_autostart(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())?;
    } else {
        mgr.disable().map_err(|e| e.to_string())?;
    }
    // Read back actual system state to confirm
    let actual = mgr.is_enabled().unwrap_or(enabled);
    let mut cfg = state.app_config.lock().unwrap();
    cfg.autostart = actual;
    cfg.save();
    sync_tray_menu(&app, &cfg);
    Ok(())
}

#[tauri::command]
fn set_quit_on_close(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut cfg = state.app_config.lock().unwrap();
    cfg.quit_on_close = enabled;
    cfg.save();
    sync_tray_menu(&app, &cfg);
    Ok(())
}

#[tauri::command]
fn set_minimize_to_tray(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut cfg = state.app_config.lock().unwrap();
    cfg.minimize_to_tray = enabled;
    cfg.save();
    sync_tray_menu(&app, &cfg);
    Ok(())
}

#[tauri::command]
fn set_locale(app: AppHandle, state: tauri::State<'_, AppState>, locale: String) {
    let mut cfg = state.app_config.lock().unwrap();
    cfg.locale = locale;
    cfg.save();
    sync_tray_menu(&app, &cfg);
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn hide_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
        let state = app.state::<AppState>();
        let cfg = state.app_config.lock().unwrap().clone();
        sync_tray_menu(&app, &cfg);
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn dismiss_reminder(id: String) {
    // Signal the scheduler to restart this reminder's interval from now.
    if let Ok(mut lock) = reminder::DISMISSED_ID.lock() {
        *lock = Some(id);
    }
}

// ── Tray helpers ──────────────────────────────────────────────────────────────

fn main_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main").map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false)
}

fn sync_tray_menu(app: &AppHandle, cfg: &AppConfig) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_tray_menu(app, cfg) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn build_tray_menu(app: &AppHandle, cfg: &AppConfig) -> tauri::Result<Menu<tauri::Wry>> {
    let visible = main_window_visible(app);
    let zh = cfg.locale == "zh";

    let toggle_label = match (visible, zh) {
        (true, true) => "隐藏主界面",
        (true, false) => "Hide Window",
        (false, true) => "显示主界面",
        (false, false) => "Show Window",
    };

    let toggle_win = MenuItem::with_id(app, "toggle_win", toggle_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        if zh { "配置" } else { "Settings" },
        true,
        None::<&str>,
    )?;
    let sep = PredefinedMenuItem::separator(app)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        if zh { "开机启动" } else { "Launch at Startup" },
        true,
        cfg.autostart,
        None::<&str>,
    )?;
    let quit_close = CheckMenuItem::with_id(
        app,
        "quit_on_close",
        if zh { "关闭时退出" } else { "Quit on Close" },
        true,
        cfg.quit_on_close,
        None::<&str>,
    )?;
    let min_tray = CheckMenuItem::with_id(
        app,
        "min_tray",
        if zh { "最小化到托盘" } else { "Minimize to Tray" },
        true,
        cfg.minimize_to_tray,
        None::<&str>,
    )?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit =
        MenuItem::with_id(app, "quit", if zh { "退出" } else { "Quit" }, true, None::<&str>)?;

    Menu::with_items(
        app,
        &[&toggle_win, &settings, &sep, &autostart, &quit_close, &min_tray, &sep2, &quit],
    )
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_config = AppConfig::load();

    let state = AppState {
        reminder_manager: Mutex::new(ReminderManager::load().unwrap_or_default()),
        app_config: Mutex::new(app_config),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_reminders,
            get_countdowns,
            save_reminder,
            delete_reminder,
            toggle_reminder,
            get_app_config,
            set_autostart,
            set_quit_on_close,
            set_minimize_to_tray,
            set_locale,
            show_main_window,
            hide_main_window,
            quit_app,
            dismiss_reminder,
        ])
        .setup(|app| {
            // Sync autostart state from system (may differ from saved config)
            {
                let state = app.state::<AppState>();
                let mut cfg = state.app_config.lock().unwrap();
                let system_autostart = app.autolaunch().is_enabled().unwrap_or(false);
                if cfg.autostart != system_autostart {
                    cfg.autostart = system_autostart;
                    cfg.save();
                }
            }

            let cfg = app.state::<AppState>().app_config.lock().unwrap().clone();

            // Build tray with menu
            let menu = build_tray_menu(app.handle(), &cfg)?;
            let tray = TrayIconBuilder::with_id("main")
                .tooltip("TimeVeil")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let cfg = app.state::<AppState>().app_config.lock().unwrap().clone();
                    match event.id.as_ref() {
                        "toggle_win" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let visible = win.is_visible().unwrap_or(false);
                                if visible {
                                    let _ = win.hide();
                                } else {
                                    let _ = win.unminimize();
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                }
                                sync_tray_menu(app, &cfg);
                            }
                        },
                        "settings" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.unminimize();
                                let _ = win.show();
                                let _ = win.set_focus();
                                let _ = win.emit("open-settings", ());
                            }
                        },
                        "autostart" => {
                            let new_val = !cfg.autostart;
                            let _ = set_autostart_inner(app, new_val);
                        },
                        "quit_on_close" => {
                            let new_val = !cfg.quit_on_close;
                            {
                                let state = app.state::<AppState>();
                                let mut c = state.app_config.lock().unwrap();
                                c.quit_on_close = new_val;
                                c.save();
                            }
                            let state = app.state::<AppState>();
                            let cfg2 = state.app_config.lock().unwrap().clone();
                            sync_tray_menu(app, &cfg2);
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.emit("config-changed", ());
                            }
                        },
                        "min_tray" => {
                            let new_val = !cfg.minimize_to_tray;
                            {
                                let state = app.state::<AppState>();
                                let mut c = state.app_config.lock().unwrap();
                                c.minimize_to_tray = new_val;
                                c.save();
                            }
                            let state = app.state::<AppState>();
                            let cfg2 = state.app_config.lock().unwrap().clone();
                            sync_tray_menu(app, &cfg2);
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.emit("config-changed", ());
                            }
                        },
                        "quit" => app.exit(0),
                        _ => {},
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let cfg = app.state::<AppState>().app_config.lock().unwrap().clone();
                        if let Some(win) = app.get_webview_window("main") {
                            let visible = win.is_visible().unwrap_or(false);
                            if visible {
                                let _ = win.hide();
                            } else {
                                let _ = win.unminimize();
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                            sync_tray_menu(app, &cfg);
                        }
                    }
                })
                .build(app)?;
            drop(tray);

            // Start scheduler
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(reminder::start_scheduler(app_handle));
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let cfg = state.app_config.lock().unwrap().clone();

                if let WindowEvent::CloseRequested { api, .. } = event {
                    if cfg.quit_on_close {
                        app.exit(0);
                    } else {
                        let _ = window.hide();
                        api.prevent_close();
                        sync_tray_menu(app, &cfg);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// helper to avoid borrow issues in closure
fn set_autostart_inner(app: &AppHandle, enabled: bool) {
    let mgr = app.autolaunch();
    let ok = if enabled { mgr.enable() } else { mgr.disable() };
    if ok.is_ok() {
        // Read back the actual system state to confirm
        let actual = mgr.is_enabled().unwrap_or(enabled);
        {
            let state = app.state::<AppState>();
            let mut cfg = state.app_config.lock().unwrap();
            cfg.autostart = actual;
            cfg.save();
        }
        let state = app.state::<AppState>();
        let cfg2 = state.app_config.lock().unwrap().clone();
        sync_tray_menu(app, &cfg2);
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.emit("config-changed", ());
        }
    }
}
