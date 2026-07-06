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
    path.push("time-reminder");
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

// ── Corner window layout ──────────────────────────────────────────────────────

/// Corner notification dimensions (logical pixels)
pub const CORNER_W: u32 = 340;
pub const CORNER_H: u32 = 44;
pub const CORNER_GAP: u32 = 4;
pub const CORNER_MARGIN: u32 = 12;

/// Ordered list of active corner overlay window labels (bottom = index 0).
/// Each entry is (window_label, reminder_id).
pub static CORNER_STACK: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// Pending overlay payloads waiting for the window to finish loading.
/// Key = window label.
pub static OVERLAY_PENDING: std::sync::LazyLock<
    Mutex<std::collections::HashMap<String, serde_json::Value>>,
> = std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

/// Recompute Y positions and push "reposition" events to all corner windows.
fn relayout_corners(app: &tauri::AppHandle) {
    let stack = match CORNER_STACK.lock() {
        Ok(s) => s.clone(),
        Err(_) => return,
    };

    // Get work area (excludes taskbar) in physical pixels from the primary monitor.
    // Monitor::work_area() is Tauri 2's official API for this — no FFI needed.
    let monitor = if let Some(win) = app.get_webview_window("main") {
        win.primary_monitor().ok().flatten()
    } else if let Some((label, _)) = stack.first() {
        app.get_webview_window(label).and_then(|w| w.primary_monitor().ok().flatten())
    } else {
        None
    };

    let monitor = match monitor {
        Some(m) => m,
        None => return,
    };

    let scale = monitor.scale_factor();
    let wa = monitor.work_area();
    // work_area() returns &PhysicalRect with .position (PhysicalPosition) and .size (PhysicalSize)
    let wa_right = wa.position.x as i32 + wa.size.width as i32;
    let wa_bottom = wa.position.y as i32 + wa.size.height as i32;

    // Card dimensions in physical pixels
    let card_w = (CORNER_W as f64 * scale) as i32;
    let card_h = (CORNER_H as f64 * scale) as i32;
    let gap = (CORNER_GAP as f64 * scale) as i32;
    let margin = (CORNER_MARGIN as f64 * scale) as i32;

    let x = wa_right - card_w - margin;

    // index 0 = bottom-most (oldest), higher index = newer (further up)
    for (i, (label, _rid)) in stack.iter().enumerate() {
        let y = wa_bottom - margin - (i as i32 + 1) * card_h - i as i32 * gap;

        if let Some(win) = app.get_webview_window(label) {
            let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
            let _ = win.emit("reposition", serde_json::json!({ "slot": i }));
        }
    }
}

/// Create a new overlay window for one reminder firing.
pub fn spawn_overlay(app: &tauri::AppHandle, reminder: &ReminderConfig, sound_volume: u8) -> bool {
    let label = format!("overlay-{}", reminder.id);

    // If a window for this reminder is already showing, don't spawn a second one.
    if app.get_webview_window(&label).is_some() {
        return true;
    }

    let payload = serde_json::json!({
        "id":         reminder.id,
        "label":      label.clone(),
        "text":       reminder.text,
        "duration":   reminder.display_secs,
        "playSound":  reminder.play_sound,
        "fullscreen": reminder.fullscreen,
        "volume":     sound_volume,
    });

    // Stash payload so the page can fetch it once ready
    if let Ok(mut map) = OVERLAY_PENDING.lock() {
        map.insert(label.clone(), payload.clone());
    }

    // Register in corner stack before creating the window so relayout sees it
    if !reminder.fullscreen {
        if let Ok(mut stack) = CORNER_STACK.lock() {
            stack.push((label.clone(), reminder.id.clone()));
        }
    }

    // Build the window
    let win_result =
        tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::App("/overlay".into()))
            .title("")
            .visible(false)
            .resizable(false)
            .always_on_top(true)
            .decorations(false)
            .transparent(true)
            .skip_taskbar(true)
            .shadow(false)
            .build();

    match win_result {
        Ok(win) => {
            if reminder.fullscreen {
                // Don't show yet — frontend will call show_overlay once the DOM
                // is fully painted, avoiding the white-flash on WebView init.
                let _ = win.set_fullscreen(true);
            } else {
                // Size in physical pixels — consistent with PhysicalPosition in relayout_corners
                if let Ok(Some(monitor)) = win.primary_monitor() {
                    let scale = monitor.scale_factor();
                    let _ = win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: (CORNER_W as f64 * scale) as u32,
                        height: (CORNER_H as f64 * scale) as u32,
                    }));
                } else {
                    let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize {
                        width: CORNER_W as f64,
                        height: CORNER_H as f64,
                    }));
                }
                let app2 = app.clone();
                relayout_corners(&app2);
                let _ = win.show();
            }
            true
        },
        Err(e) => {
            eprintln!("[overlay] Failed to create window {label}: {e}");
            // Clean up pending and stack on failure
            if let Ok(mut map) = OVERLAY_PENDING.lock() {
                map.remove(&label);
            }
            if !reminder.fullscreen {
                if let Ok(mut stack) = CORNER_STACK.lock() {
                    stack.retain(|(l, _)| l != &label);
                }
            }
            false
        },
    }
}

/// Called when an overlay window is dismissed (by user or timer).
/// Cleans up state and re-layouts remaining corner windows.
pub fn on_overlay_closed(app: &tauri::AppHandle, label: &str, fullscreen: bool) {
    if let Ok(mut map) = OVERLAY_PENDING.lock() {
        map.remove(label);
    }
    if !fullscreen {
        if let Ok(mut stack) = CORNER_STACK.lock() {
            stack.retain(|(l, _)| l != label);
        }
        relayout_corners(app);
    }
}

fn remaining_seconds_until(due: std::time::SystemTime, now: std::time::SystemTime) -> u64 {
    match due.duration_since(now) {
        Ok(duration) => duration.as_secs() + u64::from(duration.subsec_nanos() > 0),
        Err(_) => 0,
    }
}

pub async fn start_scheduler(app: tauri::AppHandle) {
    use std::time::{Duration as StdDuration, SystemTime};
    use tokio::time::{interval, Duration, MissedTickBehavior};

    let mut next_due: HashMap<String, SystemTime> = HashMap::new();
    // Tracks reminders currently being displayed — their timer does NOT count down.
    let mut displaying: HashMap<String, SystemTime> = HashMap::new();
    let mut tick = interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tick.tick().await;
        let now = SystemTime::now();

        let reminders = {
            let state = app.state::<crate::AppState>();
            let manager = state.reminder_manager.lock().unwrap();
            manager.get_all()
        };

        // ── Handle save-triggered reset (interval changed / new reminder) ──
        let reset_id = {
            if let Ok(mut lock) = RESET_ID.lock() {
                lock.take()
            } else {
                None
            }
        };
        if let Some(id) = reset_id {
            if let Some(reminder) = reminders.iter().find(|r| r.id == id) {
                next_due.insert(id.clone(), now + StdDuration::from_secs(reminder.interval_secs));
            }
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
            if let Some(reminder) = reminders.iter().find(|r| r.id == id) {
                next_due.insert(id.clone(), now + StdDuration::from_secs(reminder.interval_secs));
            }
            displaying.remove(&id);
        }

        let sound_volume = {
            let state = app.state::<crate::AppState>();
            let v = state.app_config.lock().unwrap().sound_volume;
            v
        };

        // Clean up stale entries
        let active_ids: std::collections::HashSet<String> =
            reminders.iter().map(|r| r.id.clone()).collect();
        let reminder_map: HashMap<String, ReminderConfig> =
            reminders.iter().map(|r| (r.id.clone(), r.clone())).collect();
        next_due.retain(|id, _| active_ids.contains(id));
        let mut finished_displaying = Vec::new();
        displaying.retain(|id, started_at| {
            let Some(reminder) = reminder_map.get(id) else {
                finished_displaying.push(id.clone());
                return false;
            };
            let label = format!("overlay-{id}");
            let window_alive = app.get_webview_window(&label).is_some();
            let elapsed = now.duration_since(*started_at).unwrap_or_default().as_secs();
            let timed_out = elapsed > reminder.display_secs.saturating_add(10);

            if !window_alive || timed_out {
                if let Some(win) = app.get_webview_window(&label) {
                    let _ = win.close();
                }
                on_overlay_closed(&app, &label, reminder.fullscreen);
                finished_displaying.push(id.clone());
                false
            } else {
                true
            }
        });

        for id in finished_displaying {
            if let Some(reminder) = reminder_map.get(&id) {
                next_due.insert(id, now + StdDuration::from_secs(reminder.interval_secs));
            }
        }

        for reminder in reminders.iter().filter(|r| r.enabled) {
            if displaying.contains_key(&reminder.id) {
                continue;
            }

            let should_trigger = match next_due.get(&reminder.id) {
                Some(due) => now >= *due,
                None => {
                    next_due.insert(
                        reminder.id.clone(),
                        now + StdDuration::from_secs(reminder.interval_secs),
                    );
                    false
                },
            };

            if should_trigger {
                if spawn_overlay(&app, reminder, sound_volume) {
                    displaying.insert(reminder.id.clone(), now);
                } else {
                    next_due.insert(reminder.id.clone(), now + StdDuration::from_secs(5));
                }
            }
        }

        // Snapshot for countdowns (displaying reminders show 0)
        let snapshot: Vec<(String, u64)> = reminders
            .iter()
            .map(|r| {
                let remaining = if r.enabled {
                    if displaying.contains_key(&r.id) {
                        0
                    } else {
                        match next_due.get(&r.id) {
                            Some(due) => remaining_seconds_until(*due, now),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn reminder(id: &str, interval_secs: u64) -> ReminderConfig {
        ReminderConfig {
            id: id.to_string(),
            name: format!("Reminder {id}"),
            text: "Move".to_string(),
            interval_secs,
            display_secs: 10,
            enabled: true,
            play_sound: true,
            fullscreen: false,
        }
    }

    #[test]
    fn unit_upsert_adds_and_replaces_by_id() {
        let mut manager = ReminderManager::default();

        assert!(manager.upsert(reminder("a", 60)));
        assert_eq!(manager.reminders.len(), 1);

        let mut updated = reminder("a", 120);
        updated.enabled = false;
        assert!(!manager.upsert(updated.clone()));

        assert_eq!(manager.reminders, vec![updated]);
    }

    #[test]
    fn unit_upsert_checked_reports_only_interval_changes() {
        let mut manager = ReminderManager::default();

        assert_eq!(manager.upsert_checked(reminder("a", 60)), (true, true));

        let mut same_interval = reminder("a", 60);
        same_interval.text = "Changed text".to_string();
        assert_eq!(manager.upsert_checked(same_interval), (false, false));

        assert_eq!(manager.upsert_checked(reminder("a", 120)), (false, true));
    }

    #[test]
    fn unit_remove_and_set_enabled_ignore_missing_ids() {
        let mut manager =
            ReminderManager { reminders: vec![reminder("a", 60), reminder("b", 120)] };

        manager.set_enabled("b", false);
        manager.set_enabled("missing", false);
        assert!(!manager.reminders.iter().find(|r| r.id == "b").unwrap().enabled);

        manager.remove("a");
        manager.remove("missing");
        assert_eq!(manager.reminders.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), vec!["b"]);
    }

    #[test]
    fn unit_remaining_seconds_until_rounds_up_partial_seconds() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);

        assert_eq!(remaining_seconds_until(now + Duration::from_millis(1), now), 1);
        assert_eq!(remaining_seconds_until(now + Duration::from_secs(2), now), 2);
        assert_eq!(remaining_seconds_until(now - Duration::from_secs(1), now), 0);
    }

    #[test]
    fn integration_manager_flow_add_update_toggle_and_remove() {
        let mut manager = ReminderManager::default();

        let first = reminder("focus", 1500);
        let second = reminder("break", 300);
        assert_eq!(manager.upsert_checked(first), (true, true));
        assert_eq!(manager.upsert_checked(second), (true, true));

        manager.set_enabled("break", false);
        let break_reminder =
            manager.get_all().into_iter().find(|r| r.id == "break").expect("break reminder exists");
        assert!(!break_reminder.enabled);

        let mut updated = reminder("focus", 1800);
        updated.fullscreen = true;
        assert_eq!(manager.upsert_checked(updated.clone()), (false, true));
        assert_eq!(
            manager.get_all().into_iter().find(|r| r.id == "focus").expect("focus reminder exists"),
            updated
        );

        manager.remove("break");
        assert_eq!(manager.get_all(), vec![updated]);
    }

    #[test]
    fn unit_reminder_config_deserializes_missing_fullscreen_as_false() {
        let config: ReminderConfig = serde_json::from_str(
            r#"{
                "id": "legacy",
                "name": "Legacy",
                "text": "Move",
                "interval_secs": 60,
                "display_secs": 10,
                "enabled": true,
                "play_sound": false
            }"#,
        )
        .expect("legacy reminder config should deserialize");

        assert!(!config.fullscreen);
    }
}
