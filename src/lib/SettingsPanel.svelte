<script>
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { t } from "$lib/i18n.js";

    let { onClose } = $props();

    let autostart = $state(false);
    let quitOnClose = $state(false);
    let minimizeToTray = $state(false);
    let loading = $state(true);

    async function loadConfig() {
        const cfg = await invoke("get_app_config");
        autostart = cfg.autostart;
        quitOnClose = cfg.quit_on_close;
        minimizeToTray = cfg.minimize_to_tray;
        loading = false;
    }

    onMount(async () => {
        await loadConfig();
        const win = getCurrentWebviewWindow();
        await win.listen("config-changed", loadConfig);
    });

    async function toggleAutostart() {
        autostart = !autostart;
        await invoke("set_autostart", { enabled: autostart });
    }

    async function toggleQuitOnClose() {
        quitOnClose = !quitOnClose;
        await invoke("set_quit_on_close", { enabled: quitOnClose });
    }

    async function toggleMinimizeToTray() {
        minimizeToTray = !minimizeToTray;
        await invoke("set_minimize_to_tray", { enabled: minimizeToTray });
    }
</script>

<div class="settings">
    {#if loading}
        <div class="loading">…</div>
    {:else}
        <div class="setting-item">
            <div class="setting-text">
                <span class="setting-label">{$t.settingsAutostart}</span>
                <span class="setting-desc">{$t.settingsAutostartDesc}</span>
            </div>
            <label class="toggle-wrap">
                <input
                    type="checkbox"
                    checked={autostart}
                    onchange={toggleAutostart}
                />
                <span class="toggle-track"
                    ><span class="toggle-thumb"></span></span
                >
            </label>
        </div>

        <div class="setting-item">
            <div class="setting-text">
                <span class="setting-label">{$t.settingsQuitOnClose}</span>
                <span class="setting-desc">{$t.settingsQuitOnCloseDesc}</span>
            </div>
            <label class="toggle-wrap">
                <input
                    type="checkbox"
                    checked={quitOnClose}
                    onchange={toggleQuitOnClose}
                />
                <span class="toggle-track"
                    ><span class="toggle-thumb"></span></span
                >
            </label>
        </div>

        <div class="setting-item">
            <div class="setting-text">
                <span class="setting-label">{$t.settingsMinimizeToTray}</span>
                <span class="setting-desc">{$t.settingsMinimizeToTrayDesc}</span
                >
            </div>
            <label class="toggle-wrap">
                <input
                    type="checkbox"
                    checked={minimizeToTray}
                    onchange={toggleMinimizeToTray}
                />
                <span class="toggle-track"
                    ><span class="toggle-thumb"></span></span
                >
            </label>
        </div>
    {/if}
</div>

<style>
    .settings {
        display: flex;
        flex-direction: column;
        gap: 4px;
        padding: 8px;
        height: 100%;
    }

    .loading {
        color: var(--text-muted);
        padding: 20px;
        text-align: center;
    }

    .setting-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 14px 12px;
        border-radius: var(--radius);
        background: var(--bg-card);
        border: 1px solid var(--border);
    }

    .setting-text {
        display: flex;
        flex-direction: column;
        gap: 3px;
        flex: 1;
        min-width: 0;
    }

    .setting-label {
        font-size: 14px;
        font-weight: 500;
        color: var(--text-primary);
    }

    .setting-desc {
        font-size: 12px;
        color: var(--text-muted);
        line-height: 1.4;
    }

    .toggle-wrap {
        cursor: pointer;
        flex-shrink: 0;
    }

    .toggle-wrap input {
        display: none;
    }

    .toggle-track {
        display: block;
        width: 40px;
        height: 22px;
        background: var(--border);
        border-radius: 11px;
        position: relative;
        transition: background 0.2s;
    }

    .toggle-wrap input:checked + .toggle-track {
        background: var(--accent);
    }

    .toggle-thumb {
        position: absolute;
        top: 3px;
        left: 3px;
        width: 16px;
        height: 16px;
        background: #fff;
        border-radius: 50%;
        transition: transform 0.2s;
    }

    .toggle-wrap input:checked + .toggle-track .toggle-thumb {
        transform: translateX(18px);
    }
</style>
