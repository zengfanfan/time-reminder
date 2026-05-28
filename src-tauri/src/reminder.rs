use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

pub static RESET_ID: Mutex<Option<String>> = Mutex::new(None);
pub static LAST_COUNTDOWNS: Mutex<Option<Vec<(String, u64)>>> = Mutex::new(None);
/// Set by dismiss_reminder command; scheduler resets that reminder's timer on next tick.
pub static DISMISSED_ID: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReminderConfig {
    pub id: String,
    pub name: String,
    pub text: String,
    pub interval_secs: u64,
    pub display_secs: u64,
    pub enabled: bool,
    pub play_sound: bool,
    #[serde(default)]
    pub fullscreen: bool,
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
        if let Some(existing) = self.reminders.iter_mut().find(|r| r.id == config.id) {
            *existing = config;
            false
        } else {
            self.reminders.push(config);
            true
        }
    }

    pub fn upsert_checked(&mut self, config: ReminderConfig) -> (bool, bool) {
        if let Some(existing) = self.reminders.iter_mut().find(|r| r.id == config.id) {
            let interval_changed = existing.interval_secs != config.interval_secs;
            *existing = config;
            (false, interval_changed)
        } else {
            self.reminders.push(config);
            (true, true)
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

/// Corner notification dimensions (logical pixels)
const CORNER_W: u32 = 340;
const CORNER_H: u32 = 110;
const CORNER_MARGIN: u32 = 24;

fn setup_overlay_window(overlay: &tauri::WebviewWindow, fullscreen: bool) {
    if fullscreen {
        // Restore to fullscreen
        let _ = overlay.set_fullscreen(true);
    } else {
        // Exit fullscreen first, then resize + reposition to bottom-right corner
        let _ = overlay.set_fullscreen(false);

        // Get the monitor the cursor/primary monitor occupies
        if let Ok(Some(monitor)) = overlay.primary_monitor() {
            let screen_size = monitor.size();
            let scale = monitor.scale_factor();

            // Convert logical corner dimensions to physical pixels
            let phys_w = (CORNER_W as f64 * scale) as u32;
            let phys_h = (CORNER_H as f64 * scale) as u32;
            let margin_phys = (CORNER_MARGIN as f64 * scale) as u32;

            let x = (screen_size.width.saturating_sub(phys_w + margin_phys)) as i32;
            let y = (screen_size.height.saturating_sub(phys_h + margin_phys)) as i32;

            let _ = overlay.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: phys_w,
                height: phys_h,
            }));
            let _ =
                overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        } else {
            // Fallback: fixed logical size
            let _ = overlay.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: CORNER_W as f64,
                height: CORNER_H as f64,
            }));
        }
    }
}

pub async fn start_scheduler(app: tauri::AppHandle) {
    use tokio::time::{interval, Duration};

    let mut last_triggered: HashMap<String, tokio::time::Instant> = HashMap::new();
    // Tracks reminders currently being displayed — their timer does NOT count down.
    let mut displaying: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tick.tick().await;

        // ── Handle save-triggered reset (interval changed / new reminder) ──
        let reset_id = {
            if let Ok(mut lock) = RESET_ID.lock() {
                lock.take()
            } else {
                None
            }
        };
        if let Some(id) = reset_id {
            last_triggered.insert(id.clone(), tokio::time::Instant::now());
            displaying.remove(&id);
        }

        // ── Handle dismiss signal from overlay ──
        let dismissed_id = {
            if let Ok(mut lock) = DISMISSED_ID.lock() {
                lock.take()
            } else {
                None
            }
        };
        if let Some(id) = dismissed_id {
            // Restart the interval from the moment the user finishes the break.
            last_triggered.insert(id.clone(), tokio::time::Instant::now());
            displaying.remove(&id);
        }

        let reminders = {
            let state = app.state::<crate::AppState>();
            let manager = state.reminder_manager.lock().unwrap();
            manager.get_all()
        };

        let now = tokio::time::Instant::now();

        // Clean up stale entries
        let active_ids: std::collections::HashSet<String> =
            reminders.iter().map(|r| r.id.clone()).collect();
        last_triggered.retain(|id, _| active_ids.contains(id));
        displaying.retain(|id| active_ids.contains(id));

        for reminder in reminders.iter().filter(|r| r.enabled) {
            // While overlay is showing, freeze the countdown for this reminder.
            if displaying.contains(&reminder.id) {
                continue;
            }

            let should_trigger = match last_triggered.get(&reminder.id) {
                Some(last) => now.duration_since(*last).as_secs() >= reminder.interval_secs,
                None => {
                    last_triggered.insert(reminder.id.clone(), now);
                    false
                },
            };

            if should_trigger {
                displaying.insert(reminder.id.clone());

                if let Some(overlay) = app.get_webview_window("overlay") {
                    // Adjust window geometry BEFORE showing
                    setup_overlay_window(&overlay, reminder.fullscreen);

                    let _ = overlay.emit(
                        "show-reminder",
                        serde_json::json!({
                            "id": reminder.id,
                            "text": reminder.text,
                            "duration": reminder.display_secs,
                            "playSound": reminder.play_sound,
                            "fullscreen": reminder.fullscreen,
                        }),
                    );
                    let _ = overlay.show();
                    if reminder.fullscreen {
                        let _ = overlay.set_focus();
                    }
                }
            }
        }

        // Snapshot for countdowns (displaying reminders show 0)
        let snapshot: Vec<(String, u64)> = reminders
            .iter()
            .map(|r| {
                let remaining = if r.enabled {
                    if displaying.contains(&r.id) {
                        0
                    } else {
                        match last_triggered.get(&r.id) {
                            Some(last) => {
                                let elapsed =
                                    now.duration_since(*last).as_secs_f64().round() as u64;
                                r.interval_secs.saturating_sub(elapsed)
                            },
                            None => r.interval_secs,
                        }
                    }
                } else {
                    0
                };
                (r.id.clone(), remaining)
            })
            .collect();

        if let Ok(mut lock) = LAST_COUNTDOWNS.lock() {
            *lock = Some(snapshot.clone());
        }

        let payload: Vec<serde_json::Value> = snapshot
            .iter()
            .map(|(id, remaining)| serde_json::json!({ "id": id, "remaining": remaining }))
            .collect();
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.emit("countdown-tick", payload);
        }
    }
}
