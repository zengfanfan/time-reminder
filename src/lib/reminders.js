import { invoke } from "@tauri-apps/api/core";

export function uid() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
}

export function createDefaultReminder(tr) {
  return {
    id: uid(),
    name: tr?.defaultName ?? "New Reminder",
    text: tr?.defaultText ?? "Time for a break! 👀",
    interval_secs: 1800,
    display_secs: 300,
    enabled: true,
    play_sound: true,
  };
}

export async function getCountdowns() {
  return await invoke("get_countdowns");
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
