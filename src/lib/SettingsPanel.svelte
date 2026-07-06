<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { t } from "$lib/i18n";
    import type { AppConfig } from "$lib/types";

    let { onClose }: { onClose?: () => void } = $props();

    let autostart = $state(false);
    let hideMainWindowOnStartup = $state(false);
    let soundVolume = $state(60);
    let loading = $state(true);

    async function loadConfig() {
        const cfg = await invoke<AppConfig>("get_app_config");
        autostart = cfg.autostart ?? false;
        hideMainWindowOnStartup = cfg.hide_main_window_on_startup ?? false;
        soundVolume = cfg.sound_volume ?? 60;
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

    async function toggleHideMainWindowOnStartup() {
        hideMainWindowOnStartup = !hideMainWindowOnStartup;
        await invoke("set_hide_main_window_on_startup", {
            enabled: hideMainWindowOnStartup,
        });
    }

    async function handleVolumeChange(e: Event) {
        const target = e.currentTarget;
        if (!(target instanceof HTMLInputElement)) return;
        soundVolume = Number(target.value);
        await invoke("set_sound_volume", { volume: soundVolume });
    }

    function previewBeep() {
        try {
            const ctx = new AudioContext();
            const vol = soundVolume / 100;
            const beep = (freq: number, delay: number) => {
                setTimeout(() => {
                    const osc = ctx.createOscillator();
                    const gain = ctx.createGain();
                    osc.connect(gain);
                    gain.connect(ctx.destination);
                    osc.frequency.value = freq;
                    osc.type = "sine";
                    gain.gain.setValueAtTime(vol * 0.5, ctx.currentTime);
                    gain.gain.exponentialRampToValueAtTime(
                        0.0001,
                        ctx.currentTime + 0.5,
                    );
                    osc.start(ctx.currentTime);
                    osc.stop(ctx.currentTime + 0.5);
                }, delay);
            };
            beep(660, 0);
            beep(880, 300);
        } catch (e) {
            console.warn("Audio playback failed:", e);
        }
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
                <span class="setting-label"
                    >{$t.settingsHideMainWindowOnStartup}</span
                >
                <span class="setting-desc"
                    >{$t.settingsHideMainWindowOnStartupDesc}</span
                >
            </div>
            <label class="toggle-wrap">
                <input
                    type="checkbox"
                    checked={hideMainWindowOnStartup}
                    onchange={toggleHideMainWindowOnStartup}
                />
                <span class="toggle-track"
                    ><span class="toggle-thumb"></span></span
                >
            </label>
        </div>

        <!-- Volume slider -->
        <div class="setting-item setting-item--column">
            <div class="setting-row-top">
                <div class="setting-text">
                    <span class="setting-label">{$t.settingsSoundVolume}</span>
                    <span class="setting-desc"
                        >{$t.settingsSoundVolumeDesc}</span
                    >
                </div>
                <div class="volume-right">
                    <span class="volume-value">{soundVolume}%</span>
                    <button class="btn-preview" onclick={previewBeep}>🔊</button
                    >
                </div>
            </div>
            <div class="slider-wrap">
                <span class="slider-icon">🔈</span>
                <input
                    class="slider"
                    type="range"
                    min="1"
                    max="100"
                    step="1"
                    value={soundVolume}
                    style="--pct: {soundVolume}%"
                    onchange={handleVolumeChange}
                    oninput={handleVolumeChange}
                />
                <span class="slider-icon">🔊</span>
            </div>
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

    .setting-item--column {
        flex-direction: column;
        align-items: stretch;
        gap: 12px;
    }

    .setting-row-top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
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

    /* volume right side */
    .volume-right {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;
    }

    .volume-value {
        font-size: 13px;
        font-weight: 600;
        font-family: var(--mono);
        color: var(--accent);
        min-width: 40px;
        text-align: right;
    }

    .btn-preview {
        background: var(--bg-input);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 14px;
        cursor: pointer;
        transition: all 0.15s;
        line-height: 1;
    }
    .btn-preview:hover {
        border-color: var(--accent);
        background: var(--accent-soft);
    }

    /* slider row */
    .slider-wrap {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .slider-icon {
        font-size: 14px;
        flex-shrink: 0;
        user-select: none;
    }

    .slider {
        flex: 1;
        -webkit-appearance: none;
        appearance: none;
        height: 4px;
        border-radius: 2px;
        background: linear-gradient(
            to right,
            var(--accent) 0%,
            var(--accent) calc(var(--pct, 59%)),
            var(--border) calc(var(--pct, 59%)),
            var(--border) 100%
        );
        outline: none;
        cursor: pointer;
    }

    .slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: #fff;
        box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
        cursor: pointer;
        transition: box-shadow 0.15s;
    }
    .slider::-webkit-slider-thumb:hover {
        box-shadow:
            0 0 0 4px var(--accent-soft),
            0 1px 4px rgba(0, 0, 0, 0.4);
    }

    .slider::-moz-range-thumb {
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: #fff;
        border: none;
        box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
        cursor: pointer;
    }

    /* toggle */
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
