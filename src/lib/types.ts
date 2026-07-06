export interface ReminderConfig {
  id: string;
  name: string;
  text: string;
  interval_secs: number;
  display_secs: number;
  enabled: boolean;
  play_sound: boolean;
  fullscreen: boolean;
}

export interface CountdownItem {
  id: string;
  remaining: number;
}

export type CountdownMap = Record<string, number>;

export interface AppConfig {
  autostart?: boolean;
  hide_main_window_on_startup?: boolean;
  sound_volume?: number;
}

export type LocaleCode = "zh" | "en";

export interface Translations {
  appName: string;
  newReminder: string;
  editReminder: string;
  titlePlaceholder: string;
  loading: string;
  emptyTitle: string;
  emptyHint: string;
  addReminder: string;
  everyLabel: string;
  remindOnceLabel: string;
  textPlaceholder: string;
  playSound: string;
  fullscreenLabel: string;
  fullscreenDesc: string;
  listenBtn: string;
  preview: string;
  createBtn: string;
  saveBtn: string;
  deleteBtn: string;
  deleteConfirm: string;
  deleteConfirmHint: string;
  confirmYes: string;
  confirmNo: string;
  defaultName: string;
  defaultText: string;
  dismiss: string;
  dismissHint: string;
  upcoming: string;
  settings: string;
  switchLanguage: string;
  settingsAutostart: string;
  settingsAutostartDesc: string;
  settingsHideMainWindowOnStartup: string;
  settingsHideMainWindowOnStartupDesc: string;
  settingsSoundVolume: string;
  settingsSoundVolumeDesc: string;
  unitSeconds: string;
  unitMinutes: string;
  unitHours: string;
  countdownH: (h: number, m: number) => string;
  countdownM: (m: number, s: number) => string;
  countdownS: (s: number) => string;
  metaEvery: (dur: string) => string;
  metaDisplay: (dur: string) => string;
  durationH: (h: number, m: number) => string;
  durationM: (m: number, s: number) => string;
  durationS: (s: number) => string;
}

export interface OverlayData {
  id?: string;
  text?: string;
  duration?: number;
  fullscreen?: boolean;
  volume?: number;
  playSound?: boolean;
}
