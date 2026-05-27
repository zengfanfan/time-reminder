use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

/// Holds the id of the reminder whose timer should be reset, or None if no reset needed.
/// "NEW" is a special sentinel meaning a brand-new reminder was added.
pub static RESET_ID: Mutex<Option<String>> = Mutex::new(None);

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

    pub fn save_silent(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn save(&self, changed_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        if let Ok(mut lock) = RESET_ID.lock() {
            *lock = Some(changed_id.to_string());
        }
        Ok(())
    }

    pub fn get_all(&self) -> Vec<ReminderConfig> {
        self.reminders.clone()
    }

    pub fn upsert(&mut self, config: ReminderConfig) -> bool {
        // Returns true if this is a new reminder (not an update)
        if let Some(existing) = self.reminders.iter_mut().find(|r| r.id == config.id) {
            *existing = config;
            false
        } else {
            self.reminders.push(config);
            true
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

        // Check if a specific reminder's timer needs resetting
        let reset_id = {
            if let Ok(mut lock) = RESET_ID.lock() {
                lock.take()
            } else {
                None
            }
        };

        if let Some(id) = reset_id {
            // Reset only this reminder's timer
            last_triggered.insert(id, tokio::time::Instant::now());
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
                    // First time seeing this reminder — record time, don't fire
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

        // Broadcast remaining seconds for each reminder to the main window
        let countdown_payload: Vec<serde_json::Value> = reminders
            .iter()
            .map(|r| {
                let remaining = if r.enabled {
                    match last_triggered.get(&r.id) {
                        Some(last) => {
                            let elapsed = now.duration_since(*last).as_secs();
                            r.interval_secs.saturating_sub(elapsed)
                        },
                        None => r.interval_secs,
                    }
                } else {
                    0
                };
                serde_json::json!({ "id": r.id, "remaining": remaining })
            })
            .collect();

        if let Some(main) = app.get_webview_window("main") {
            let _ = main.emit("countdown-tick", countdown_payload);
        }
    }
}
