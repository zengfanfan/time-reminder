<script>
  import { onMount, tick } from "svelte";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import {
    loadReminders,
    saveReminder,
    deleteReminder,
    toggleReminder,
    createDefaultReminder,
  } from "$lib/reminders.js";
  import {
    locale,
    t,
    initLocale,
    toggleLocale,
    formatDurationLocale,
    formatCountdownLocale,
  } from "$lib/i18n.js";
  import ReminderEditor from "$lib/ReminderEditor.svelte";
  import SettingsPanel from "$lib/SettingsPanel.svelte";

  let reminders = $state([]);
  let editing = $state(null);
  let editingName = $state("");
  let isNew = $state(false);
  let nameInput;
  let triggerSave = $state(0);
  let countdowns = $state({});
  let showSettings = $state(false);

  onMount(async () => {
    initLocale();
    loadReminders().then((data) => {
      reminders = data;
    });

    const win = getCurrentWebviewWindow();
    // Show window after content is ready to avoid WebView2 white flash
    const cfg = await invoke("get_app_config");
    if (!cfg.hide_main_window_on_startup) {
      win.show();
    }

    // Tray menu can open settings
    await win.listen("open-settings", () => {
      showSettings = true;
      editing = null;
    });

    // Minimize to tray
    async function checkMinimized() {
      const minimized = await win.isMinimized();
      if (minimized) {
        const cfg = await invoke("get_app_config");
        if (cfg.minimize_to_tray) {
          await invoke("hide_main_window");
        }
      }
    }
    await win.listen("tauri://blur", checkMinimized);
    await win.onResized(checkMinimized);

    // Rust broadcasts every second — directly drive countdowns from it
    await win.listen("countdown-tick", (event) => {
      console.log("[tick]", Date.now(), JSON.stringify(event.payload));
      const map = {};
      for (const item of event.payload) map[item.id] = item.remaining;
      countdowns = map;
    });
  });

  function handleKeydown(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (editing) handleBack();
      else if (showSettings) showSettings = false;
    } else if (editing && e.key === "Enter") {
      const tag = e.target.tagName;
      if (tag !== "TEXTAREA" && tag !== "SELECT") {
        e.preventDefault();
        triggerSave++;
      }
    }
  }

  async function handleAdd() {
    isNew = true;
    editing = createDefaultReminder(get(t));
    editingName = editing.name;
    await tick();
    nameInput?.focus();
    nameInput?.select();
  }

  async function handleSave(config) {
    await saveReminder({ ...config, name: editingName });
    reminders = await loadReminders();
    editing = null;
    isNew = false;
  }

  async function handleDelete(id) {
    await deleteReminder(id);
    reminders = await loadReminders();
    editing = null;
    isNew = false;
  }

  async function handleToggle(id, enabled) {
    // Immediately clear the countdown for this id to prevent flash of stale value
    const { [id]: _, ...rest } = countdowns;
    countdowns = rest;
    await toggleReminder(id, enabled);
    reminders = await loadReminders();
  }

  async function handleEdit(reminder) {
    isNew = false;
    editing = { ...reminder };
    editingName = reminder.name;
    await tick();
    nameInput?.focus();
    nameInput?.select();
  }

  function handleBack() {
    editing = null;
    isNew = false;
    showSettings = false;
  }

  async function minimizeWindow() {
    await getCurrentWebviewWindow().minimize();
  }

  async function closeWindow() {
    await getCurrentWebviewWindow().close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="panel">
  <header class="topbar" data-tauri-drag-region>
    {#if editing}
      <button class="btn-icon" onclick={handleBack}>
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M19 12H5M12 19l-7-7 7-7" />
        </svg>
      </button>
      <span class="topbar-label"
        >{isNew ? $t.newReminder : $t.editReminder}</span
      >
      <input
        bind:this={nameInput}
        class="topbar-name-input"
        type="text"
        bind:value={editingName}
        placeholder={$t.titlePlaceholder}
      />
    {:else if showSettings}
      <button class="btn-icon" onclick={handleBack}>
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M19 12H5M12 19l-7-7 7-7" />
        </svg>
      </button>
      <h1>{$t.settings}</h1>
      <div class="topbar-spacer"></div>
    {:else}
      <div class="logo">
        <svg
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--accent)"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M12 6v6l4 2" />
        </svg>
        <h1>{$t.appName}</h1>
      </div>
      <div class="topbar-spacer"></div>
      <button class="btn-lang" onclick={toggleLocale} title={$t.switchLanguage}>
        {$locale === "zh" ? "EN" : "中"}
      </button>
      <button
        class="btn-icon"
        onclick={() => {
          showSettings = !showSettings;
          editing = null;
        }}
        aria-label={$t.settings}
      >
        <svg
          width="17"
          height="17"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
      </button>
    {/if}
    <div class="window-controls">
      <button
        class="window-control"
        onclick={minimizeWindow}
        aria-label={$locale === "zh" ? "最小化" : "Minimize"}
        title={$locale === "zh" ? "最小化" : "Minimize"}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <path d="M6 12h12" />
        </svg>
      </button>
      <button
        class="window-control close"
        onclick={closeWindow}
        aria-label={$locale === "zh" ? "关闭" : "Close"}
        title={$locale === "zh" ? "关闭" : "Close"}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
        >
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
  </header>

  <main class="content">
    {#if editing}
      <ReminderEditor
        config={editing}
        {isNew}
        name={editingName}
        {triggerSave}
        onSave={handleSave}
        onDelete={handleDelete}
      />
    {:else if showSettings}
      <SettingsPanel onClose={() => (showSettings = false)} />
    {:else if reminders.length === 0}
      <div class="empty">
        <div class="empty-icon">⏰</div>
        <p>{$t.emptyTitle}</p>
        <p class="muted">{$t.emptyHint}</p>
      </div>
    {:else}
      <ul class="reminder-list">
        {#each reminders as r (r.id)}
          <li class="reminder-card" class:disabled={!r.enabled}>
            <button class="card-body" onclick={() => handleEdit(r)}>
              <div class="card-info">
                <span class="card-name">{r.name}</span>
                <span class="card-meta">
                  {$t.metaEvery(formatDurationLocale(r.interval_secs, $t))} · {$t.metaDisplay(
                    formatDurationLocale(r.display_secs, $t),
                  )}
                  {#if r.play_sound}· 🔔{/if}
                </span>
              </div>
              <div class="card-bottom">
                <div class="card-preview">"{r.text}"</div>
                {#if r.enabled}
                  {@const cd = countdowns[r.id]}
                  {#if cd !== undefined && cd >= 1 && cd <= 3599}
                    <span class="card-countdown"
                      >{formatCountdownLocale(cd, $t)}</span
                    >
                  {/if}
                {/if}
              </div>
            </button>
            <label class="toggle-wrap">
              <input
                type="checkbox"
                checked={r.enabled}
                onchange={(e) => handleToggle(r.id, e.target.checked)}
              />
              <span class="toggle-track"
                ><span class="toggle-thumb"></span></span
              >
            </label>
          </li>
        {/each}
      </ul>
    {/if}
  </main>

  {#if !editing && !showSettings}
    <footer class="bottombar">
      <button class="btn-add" onclick={handleAdd}>
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path d="M12 5v14M5 12h14" />
        </svg>
        {$t.addReminder}
      </button>
    </footer>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent;
  }

  .panel {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-main);
    border: 1px solid rgba(99, 122, 190, 0.22);
    border-radius: 14px;
    overflow: hidden;
  }

  .topbar {
    padding: 12px 12px 12px 18px;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 58px;
    background:
      linear-gradient(135deg, rgba(78, 123, 255, 0.2), rgba(52, 211, 153, 0.08)),
      #151a28;
    border-bottom: 1px solid rgba(78, 123, 255, 0.28);
    -webkit-app-region: drag;
  }

  .topbar h1 {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .topbar-spacer {
    flex: 1;
  }

  .window-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    padding-left: 16px;
    margin-left: 6px;
    border-left: 1px dashed rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
    -webkit-app-region: no-drag;
  }

  .topbar-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .topbar-name-input {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    padding: 4px 2px;
    color: var(--text-primary);
    font-size: 15px;
    font-weight: 600;
    font-family: var(--sans);
    outline: none;
    -webkit-app-region: no-drag;
    transition: border-color 0.15s;
  }
  .topbar-name-input:focus {
    border-bottom-color: var(--border-focus);
  }
  .topbar-name-input::placeholder {
    color: var(--text-muted);
    font-weight: 400;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .btn-icon {
    background: rgba(255, 255, 255, 0.03);
    border: none;
    color: #b8c0df;
    cursor: pointer;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    -webkit-app-region: no-drag;
  }
  .btn-icon:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  .btn-lang {
    -webkit-app-region: no-drag;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    height: 30px;
    padding: 0 10px;
    color: #c9d1ef;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--sans);
    cursor: pointer;
    letter-spacing: 0.04em;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .btn-lang:hover {
    background: rgba(78, 123, 255, 0.16);
    border-color: rgba(78, 123, 255, 0.42);
    color: #eef;
  }

  .window-control {
    width: 32px;
    height: 30px;
    border: none;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    color: #aeb8d8;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    -webkit-app-region: no-drag;
  }
  .window-control:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }
  .window-control.close:hover {
    background: var(--danger);
    color: #fff;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
  }

  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-secondary);
  }
  .empty-icon {
    font-size: 48px;
    margin-bottom: 8px;
  }
  .muted {
    color: var(--text-muted);
    font-size: 13px;
  }

  .reminder-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .reminder-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }
  .reminder-card:hover {
    background: var(--bg-card-hover);
    border-color: var(--border-focus);
  }
  .reminder-card.disabled {
    opacity: 0.5;
  }

  .card-body {
    flex: 1;
    min-width: 0;
    padding: 14px 16px;
    text-align: left;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .card-info {
    display: flex;
    align-items: baseline;
    gap: 10px;
    overflow: hidden;
  }
  .card-name {
    font-weight: 600;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }
  .card-meta {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 2;
    min-width: 0;
  }

  .card-bottom {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .card-preview {
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .card-countdown {
    font-size: 12px;
    font-family: var(--mono);
    color: #3b5;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .toggle-wrap {
    padding: 14px 16px;
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

  .bottombar {
    padding: 12px;
    border-top: 1px solid var(--border);
  }
  .btn-add {
    width: 100%;
    padding: 12px;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    transition: all 0.15s;
    font-family: var(--sans);
  }
  .btn-add:hover {
    filter: brightness(1.15);
    box-shadow: 0 0 20px var(--accent-glow);
  }
</style>
