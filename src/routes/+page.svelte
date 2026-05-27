<script>
  import { onMount, tick } from "svelte";
  import { get } from "svelte/store";
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

  let reminders = $state([]);
  let editing = $state(null);
  let editingName = $state("");
  let isNew = $state(false);
  let loading = $state(true);
  let nameInput;
  let triggerSave = $state(0);
  let countdowns = $state({});

  onMount(async () => {
    initLocale();
    reminders = await loadReminders();
    loading = false;

    const win = getCurrentWebviewWindow();
    await win.listen("countdown-tick", (event) => {
      const map = {};
      for (const item of event.payload) map[item.id] = item.remaining;
      countdowns = map;
    });
  });

  function handleKeydown(e) {
    if (!editing) return;
    const tag = e.target.tagName;
    if (e.key === "Escape") {
      e.preventDefault();
      handleBack();
    } else if (e.key === "Enter" && tag !== "TEXTAREA" && tag !== "SELECT") {
      e.preventDefault();
      triggerSave++;
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
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="panel">
  <header class="topbar">
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
      <button
        class="btn-lang"
        onclick={toggleLocale}
        title="切换语言 / Switch language"
      >
        {$locale === "zh" ? "EN" : "中"}
      </button>
    {/if}
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
    {:else if loading}
      <div class="empty"><p class="muted">{$t.loading}</p></div>
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
                {#if r.enabled && countdowns[r.id] !== undefined}
                  <span class="card-countdown"
                    >{formatCountdownLocale(countdowns[r.id], $t)}</span
                  >
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

  {#if !editing}
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
  .panel {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-main);
  }

  .topbar {
    padding: 16px 20px;
    display: flex;
    align-items: center;
    gap: 12px;
    border-bottom: 1px solid var(--border);
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
    font-family: "Noto Sans SC", sans-serif;
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
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    display: flex;
    -webkit-app-region: no-drag;
  }
  .btn-icon:hover {
    background: var(--bg-card);
    color: var(--text-primary);
  }

  .btn-lang {
    -webkit-app-region: no-drag;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 10px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    font-family: "Noto Sans SC", sans-serif;
    cursor: pointer;
    letter-spacing: 0.04em;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .btn-lang:hover {
    border-color: var(--accent);
    color: var(--accent);
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
  }
  .card-name {
    font-weight: 600;
    font-size: 14px;
  }
  .card-meta {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--mono);
  }

  .card-bottom {
    display: flex;
    align-items: center;
    gap: 10px;
    overflow: hidden;
  }
  .card-preview {
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }
  .card-countdown {
    font-size: 12px;
    font-family: var(--mono);
    color: #2d7a4f;
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
    font-family: "Noto Sans SC", sans-serif;
  }
  .btn-add:hover {
    filter: brightness(1.15);
    box-shadow: 0 0 20px var(--accent-glow);
  }
</style>
