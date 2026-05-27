import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

const STORAGE_KEY = "time-veil-locale";

const translations = {
    zh: {
        appName: "TimeVeil",
        newReminder: "新建提醒",
        editReminder: "编辑提醒",
        titlePlaceholder: "标题",
        loading: "加载中…",
        emptyTitle: "还没有提醒",
        emptyHint: "点击下方按钮添加你的第一个提醒",
        addReminder: "添加提醒",
        everyLabel: "每",
        remindOnceLabel: "提醒一次，显示",
        textPlaceholder: "遮挡屏幕时显示的文字…",
        playSound: "提醒时播放提示音",
        listenBtn: "🔊 试听",
        preview: "预览",
        createBtn: "创建提醒",
        saveBtn: "保存修改",
        deleteBtn: "删除",
        deleteConfirm: "确认删除这个提醒？",
        deleteConfirmHint: "此操作无法撤销",
        confirmYes: "删除",
        confirmNo: "取消",
        defaultName: "新提醒",
        defaultText: "休息一下，活动活动 👀",
        dismiss: "退出",
        dismissHint: "倒计时结束后自动关闭 · 右上角可提前退出",
        upcoming: "即将提醒",
        settings: "配置",
        switchLanguage: "Switch to English",
        settingsAutostart: "开机启动",
        settingsAutostartDesc: "系统启动时自动运行 TimeVeil",
        settingsQuitOnClose: "关闭时退出",
        settingsQuitOnCloseDesc: "点击关闭按钮时退出程序，默认为隐藏到托盘",
        settingsMinimizeToTray: "最小化到托盘",
        settingsMinimizeToTrayDesc: "点击最小化按钮时隐藏到系统托盘",
        unitSeconds: "秒",
        unitMinutes: "分钟",
        unitHours: "小时",
        countdownH: (h, m) => m > 0 ? `${h}小时${m}分` : `${h}小时`,
        countdownM: (m, s) => `${m}分${s}秒`,
        countdownS: (s) => `${s}秒`,
        metaEvery: (dur) => `每 ${dur}`,
        metaDisplay: (dur) => `显示 ${dur}`,
        durationH: (h, m) => m > 0 ? `${h}小时${m}分` : `${h}小时`,
        durationM: (m, s) => s > 0 ? `${m}分${s}秒` : `${m}分钟`,
        durationS: (s) => `${s}秒`,
    },
    en: {
        appName: "TimeVeil",
        newReminder: "New Reminder",
        editReminder: "Edit Reminder",
        titlePlaceholder: "Title",
        loading: "Loading…",
        emptyTitle: "No reminders yet",
        emptyHint: "Click the button below to add your first reminder",
        addReminder: "Add Reminder",
        everyLabel: "Every",
        remindOnceLabel: "remind, show for",
        textPlaceholder: "Text shown on the fullscreen overlay…",
        playSound: "Play sound on reminder",
        listenBtn: "🔊 Preview",
        preview: "Preview",
        createBtn: "Create Reminder",
        saveBtn: "Save Changes",
        deleteBtn: "Delete",
        deleteConfirm: "Delete this reminder?",
        deleteConfirmHint: "This action cannot be undone",
        confirmYes: "Delete",
        confirmNo: "Cancel",
        defaultName: "New Reminder",
        defaultText: "Time for a break! 👀",
        dismiss: "Dismiss",
        dismissHint: "Auto-closes when countdown ends · Click top-right to dismiss early",
        upcoming: "Due soon",
        settings: "Settings",
        switchLanguage: "切换为中文",
        settingsAutostart: "Launch at startup",
        settingsAutostartDesc: "Automatically start TimeVeil when the system boots",
        settingsQuitOnClose: "Quit on close",
        settingsQuitOnCloseDesc: "Exit the app when the close button is clicked; default is hide to tray",
        settingsMinimizeToTray: "Minimize to tray",
        settingsMinimizeToTrayDesc: "Hide to system tray when the minimize button is clicked",
        unitSeconds: "sec",
        unitMinutes: "min",
        unitHours: "hr",
        countdownH: (h, m) => m > 0 ? `${h}h ${m}m` : `${h}h`,
        countdownM: (m, s) => `${m}m ${s}s`,
        countdownS: (s) => `${s}s`,
        metaEvery: (dur) => `Every ${dur}`,
        metaDisplay: (dur) => `show ${dur}`,
        durationH: (h, m) => m > 0 ? `${h}h ${m}m` : `${h}h`,
        durationM: (m, s) => s > 0 ? `${m}m ${s}s` : `${m}m`,
        durationS: (s) => `${s}s`,
    },
};

function detectLocale() {
    // Check saved preference first
    try {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved === "zh" || saved === "en") return saved;
    } catch (_) { }

    // Fall back to system language
    const lang = navigator.language || navigator.languages?.[0] || "en";
    return lang.startsWith("zh") ? "zh" : "en";
}

export const locale = writable("zh"); // default, will be set on mount

export function initLocale() {
    const l = detectLocale();
    locale.set(l);
    invoke("set_locale", { locale: l }).catch(() => { });
}

export function toggleLocale() {
    locale.update((l) => {
        const next = l === "zh" ? "en" : "zh";
        try { localStorage.setItem(STORAGE_KEY, next); } catch (_) { }
        invoke("set_locale", { locale: next }).catch(() => { });
        return next;
    });
}

export const t = derived(locale, ($locale) => translations[$locale] || translations.zh);

// Utility: format duration using current locale translations
export function formatDurationLocale(secs, tr) {
    if (secs < 60) return tr.durationS(secs);
    if (secs < 3600) {
        const m = Math.floor(secs / 60);
        const s = secs % 60;
        return tr.durationM(m, s);
    }
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return tr.durationH(h, m);
}

export function formatCountdownLocale(secs, _tr) {
    if (secs >= 3600) return null; // hide when over 1 hour
    const s = Math.max(0, secs);
    const m = Math.floor(s / 60);
    const ss = s % 60;
    return `${m.toString().padStart(2, "0")}:${ss.toString().padStart(2, "0")}`;
}
