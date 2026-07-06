import { invoke } from "@tauri-apps/api/core";
import type { CountdownMap, ReminderConfig, Translations } from "$lib/types";

export function uid(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
}

export function createDefaultReminder(
  tr?: Pick<Translations, "defaultName" | "defaultText">,
): ReminderConfig {
  return {
    id: uid(),
    name: tr?.defaultName ?? "New Reminder",
    text: tr?.defaultText ?? "Time for a break! 👀",
    interval_secs: 1800,
    display_secs: 300,
    enabled: true,
    play_sound: true,
    fullscreen: false,
  };
}

export async function getCountdowns(): Promise<CountdownMap> {
  return await invoke<CountdownMap>("get_countdowns");
}

export async function loadReminders(): Promise<ReminderConfig[]> {
  return await invoke<ReminderConfig[]>("get_reminders");
}

export async function saveReminder(config: ReminderConfig): Promise<unknown> {
  return await invoke("save_reminder", { config });
}

export async function deleteReminder(id: string): Promise<unknown> {
  return await invoke("delete_reminder", { id });
}

export async function toggleReminder(
  id: string,
  enabled: boolean,
): Promise<unknown> {
  return await invoke("toggle_reminder", { id, enabled });
}
