import { invoke } from "@tauri-apps/api/core";

export function uid() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
}

export function createDefaultReminder() {
  return {
    id: uid(),
    name: "新提醒",
    text: "休息一下，活动活动 👀",
    interval_secs: 1800,
    display_secs: 300, // 5 min
    enabled: true,
    play_sound: true,
  };
}

export async function loadReminders() {
  return await invoke("get_reminders");
}

export async function saveReminder(config) {
  return await invoke("save_reminder", { config });
}

export async function deleteReminder(id) {
  return await invoke("delete_reminder", { id });
}

export async function toggleReminder(id, enabled) {
  return await invoke("toggle_reminder", { id, enabled });
}

export function formatDuration(secs) {
  if (secs < 60) return `${secs}秒`;
  if (secs < 3600) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return s > 0 ? `${m}分${s}秒` : `${m}分钟`;
  }
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return m > 0 ? `${h}小时${m}分` : `${h}小时`;
}
