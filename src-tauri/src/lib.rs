mod reminder;

use reminder::ReminderManager;
use std::sync::Mutex;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

pub struct AppState {
    pub reminder_manager: Mutex<ReminderManager>,
}

#[tauri::command]
fn get_reminders(state: tauri::State<'_, AppState>) -> Vec<reminder::ReminderConfig> {
    let manager = state.reminder_manager.lock().unwrap();
    manager.get_all()
}

#[tauri::command]
fn save_reminder(
    state: tauri::State<'_, AppState>,
    config: reminder::ReminderConfig,
) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    manager.upsert(config);
    manager.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_reminder(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    manager.remove(&id);
    manager.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_reminder(
    state: tauri::State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.reminder_manager.lock().unwrap();
    manager.set_enabled(&id, enabled);
    manager.save().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state =
        AppState { reminder_manager: Mutex::new(ReminderManager::load().unwrap_or_default()) };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_reminders,
            save_reminder,
            delete_reminder,
            toggle_reminder,
        ])
        .setup(|app| {
            // Tray icon
            let _tray = TrayIconBuilder::new()
                .tooltip("TimeVeil")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Start scheduler in a dedicated thread with its own Tokio runtime
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(reminder::start_scheduler(app_handle));
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
