use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

/// Global flag: set to true when reminders are saved/deleted, so scheduler resets timers
pub static REMINDERS_CHANGED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub id: String,
    pub name: String,
    pub text: String,
    pub interval_secs: u64,
    pub display_secs: u64,
    pub enabled: bool,
    pub play_sound: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ReminderManager {
    pub reminders: Vec<ReminderConfig>,
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("time-veil");
    fs::create_dir_all(&path).ok();
    path.push("reminders.json");
    path
}

impl ReminderManager {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            let manager: ReminderManager = serde_json::from_str(&data)?;
            Ok(manager)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        // Signal scheduler to reset timers
        REMINDERS_CHANGED.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_all(&self) -> Vec<ReminderConfig> {
        self.reminders.clone()
    }

    pub fn upsert(&mut self, config: ReminderConfig) {
        if let Some(existing) = self.reminders.iter_mut().find(|r| r.id == config.id) {
            *existing = config;
        } else {
            self.reminders.push(config);
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.reminders.retain(|r| r.id != id);
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(r) = self.reminders.iter_mut().find(|r| r.id == id) {
            r.enabled = enabled;
        }
    }
}

pub async fn start_scheduler(app: tauri::AppHandle) {
    use std::collections::HashMap;
    use tokio::time::{interval, Duration};

    let mut last_triggered: HashMap<String, tokio::time::Instant> = HashMap::new();
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tick.tick().await;

        // If reminders changed, reset all timers so nothing fires immediately
        if REMINDERS_CHANGED.swap(false, Ordering::Relaxed) {
            let now = tokio::time::Instant::now();
            let reminders = {
                let state = app.state::<crate::AppState>();
                let manager = state.reminder_manager.lock().unwrap();
                manager.get_all()
            };
            // Reset all timers to "just triggered" so they wait a full interval
            for r in &reminders {
                last_triggered.insert(r.id.clone(), now);
            }
            continue;
        }

        let reminders = {
            let state = app.state::<crate::AppState>();
            let manager = state.reminder_manager.lock().unwrap();
            manager.get_all()
        };

        let now = tokio::time::Instant::now();

        // Clean up timers for deleted reminders
        let active_ids: std::collections::HashSet<String> =
            reminders.iter().map(|r| r.id.clone()).collect();
        last_triggered.retain(|id, _| active_ids.contains(id));

        for reminder in reminders.iter().filter(|r| r.enabled) {
            let should_trigger = match last_triggered.get(&reminder.id) {
                Some(last) => now.duration_since(*last).as_secs() >= reminder.interval_secs,
                None => {
                    // First time seeing this reminder — don't fire, just record the time
                    last_triggered.insert(reminder.id.clone(), now);
                    false
                },
            };

            if should_trigger {
                last_triggered.insert(reminder.id.clone(), now);

                if let Some(overlay) = app.get_webview_window("overlay") {
                    let _ = overlay.emit(
                        "show-reminder",
                        serde_json::json!({
                            "text": reminder.text,
                            "duration": reminder.display_secs,
                            "playSound": reminder.play_sound,
                        }),
                    );
                    let _ = overlay.show();
                    let _ = overlay.set_focus();

                    let app_clone = app.clone();
                    let dur = reminder.display_secs;
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(dur)).await;
                        if let Some(win) = app_clone.get_webview_window("overlay") {
                            let _ = win.hide();
                        }
                    });
                }
            }
        }
    }
}
