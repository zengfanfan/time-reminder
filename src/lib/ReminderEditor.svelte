<script>
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n.js";

  let { config, isNew, onSave, onDelete, name, onNameChange, triggerSave } =
    $props();

  let text = $state("");
  let playSound = $state(false);
  let fullscreen = $state(false);

  let intervalValue = $state(1);
  let intervalUnit = $state("minutes");

  let displayValue = $state(1);
  let displayUnit = $state("seconds");
  /** @type {"interval" | "display" | null} */
  let openUnitMenu = $state(null);

  /** @type {Array<{ value: "seconds" | "minutes" | "hours", labelKey: "unitSeconds" | "unitMinutes" | "unitHours" }>} */
  const unitOptions = [
    { value: "seconds", labelKey: "unitSeconds" },
    { value: "minutes", labelKey: "unitMinutes" },
    { value: "hours", labelKey: "unitHours" },
  ];

  function deriveUnit(secs) {
    if (secs >= 3600 && secs % 3600 === 0) return "hours";
    if (secs >= 60 && secs % 60 === 0) return "minutes";
    return "seconds";
  }

  function deriveValue(secs) {
    if (secs >= 3600 && secs % 3600 === 0) return secs / 3600;
    if (secs >= 60 && secs % 60 === 0) return secs / 60;
    return secs;
  }

  function toSeconds(value, unit) {
    if (unit === "hours") return value * 3600;
    if (unit === "minutes") return value * 60;
    return value;
  }

  function displaySeconds() {
    return toSeconds(displayValue, displayUnit);
  }

  function formatPreviewTimer(secs) {
    if (secs >= 3600) {
      const h = Math.floor(secs / 3600);
      const m = Math.floor((secs % 3600) / 60);
      const s = secs % 60;
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
    if (secs >= 60) {
      const m = Math.floor(secs / 60);
      const s = secs % 60;
      return `${m}:${s.toString().padStart(2, "0")}`;
    }
    return `${secs}`;
  }

  /** @type {number | null} */
  let prevTriggerSave = $state(null);
  $effect(() => {
    const currentTriggerSave = triggerSave;
    if (prevTriggerSave === null) {
      prevTriggerSave = currentTriggerSave;
      return;
    }
    if (currentTriggerSave > prevTriggerSave) {
      prevTriggerSave = currentTriggerSave;
      handleSubmit();
    }
  });

  $effect(() => {
    text = config.text;
    playSound = config.play_sound;
    fullscreen = config.fullscreen ?? false;
    intervalValue = deriveValue(config.interval_secs);
    intervalUnit = deriveUnit(config.interval_secs);
    displayValue = deriveValue(config.display_secs);
    displayUnit = deriveUnit(config.display_secs);
  });

  function handleSubmit() {
    onSave({
      ...config,
      name,
      text,
      interval_secs: toSeconds(intervalValue, intervalUnit),
      display_secs: toSeconds(displayValue, displayUnit),
      play_sound: playSound,
      fullscreen,
      enabled: config.enabled,
    });
  }

  let showDeleteConfirm = $state(false);

  function handleDelete() {
    showDeleteConfirm = true;
  }

  function confirmDelete() {
    showDeleteConfirm = false;
    onDelete(config.id);
  }

  function cancelDelete() {
    showDeleteConfirm = false;
  }

  function handleDialogKeydown(e) {
    if (!showDeleteConfirm) return;
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      confirmDelete();
    }
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      cancelDelete();
    }
  }

  /** @param {MouseEvent} e */
  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) cancelDelete();
  }

  /** @param {KeyboardEvent} e */
  function handleBackdropKeydown(e) {
    if (e.target !== e.currentTarget) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      cancelDelete();
    }
  }

  /** @param {"interval" | "display"} menu */
  function toggleUnitMenu(menu) {
    openUnitMenu = openUnitMenu === menu ? null : menu;
  }

  function closeUnitMenu() {
    openUnitMenu = null;
  }

  /**
   * @param {KeyboardEvent} e
   * @param {"interval" | "display"} menu
   */
  function handleUnitKeydown(e, menu) {
    if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
      e.preventDefault();
      openUnitMenu = menu;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      closeUnitMenu();
    }
  }

  /** @param {KeyboardEvent} e */
  function handleWindowKeydown(e) {
    if (e.key === "Escape") closeUnitMenu();
  }

  async function playBeep() {
    try {
      // Read current volume from backend so preview matches the real sound
      let vol = 0.6;
      try {
        const cfg = await invoke("get_app_config");
        vol = (cfg.sound_volume ?? 60) / 100;
      } catch (_) {}

      const ctx = new AudioContext();
      const beep = (freq, delay) => {
        setTimeout(() => {
          const osc = ctx.createOscillator();
          const gain = ctx.createGain();
          osc.connect(gain);
          gain.connect(ctx.destination);
          osc.frequency.value = freq;
          osc.type = "sine";
          gain.gain.setValueAtTime(vol * 0.5, ctx.currentTime);
          gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.5);
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

<svelte:window
  onkeydowncapture={handleDialogKeydown}
  onkeydown={handleWindowKeydown}
  onclick={closeUnitMenu}
/>

<div class="editor">
  <textarea
    class="text-input"
    bind:value={text}
    placeholder={$t.textPlaceholder}
  ></textarea>

  <div class="timing-row">
    <span class="timing-label">{$t.everyLabel}</span>
    <input
      class="timing-num"
      type="number"
      bind:value={intervalValue}
      min="1"
    />
    <div class="unit-select">
      <button
        type="button"
        class="timing-unit"
        aria-haspopup="listbox"
        aria-expanded={openUnitMenu === "interval"}
        onclick={(e) => {
          e.stopPropagation();
          toggleUnitMenu("interval");
        }}
        onkeydown={(e) => handleUnitKeydown(e, "interval")}
      >
        {$t[
          unitOptions.find((option) => option.value === intervalUnit)?.labelKey
        ]}
      </button>
      {#if openUnitMenu === "interval"}
        <div class="unit-menu" role="listbox">
          {#each unitOptions as option}
            <button
              type="button"
              class="unit-option"
              class:unit-option--selected={intervalUnit === option.value}
              role="option"
              aria-selected={intervalUnit === option.value}
              onclick={() => {
                intervalUnit = option.value;
                closeUnitMenu();
              }}
            >
              {$t[option.labelKey]}
            </button>
          {/each}
        </div>
      {/if}
    </div>
    <span class="timing-label">{$t.remindOnceLabel}</span>
    <input class="timing-num" type="number" bind:value={displayValue} min="1" />
    <div class="unit-select">
      <button
        type="button"
        class="timing-unit"
        aria-haspopup="listbox"
        aria-expanded={openUnitMenu === "display"}
        onclick={(e) => {
          e.stopPropagation();
          toggleUnitMenu("display");
        }}
        onkeydown={(e) => handleUnitKeydown(e, "display")}
      >
        {$t[
          unitOptions.find((option) => option.value === displayUnit)?.labelKey
        ]}
      </button>
      {#if openUnitMenu === "display"}
        <div class="unit-menu" role="listbox">
          {#each unitOptions as option}
            <button
              type="button"
              class="unit-option"
              class:unit-option--selected={displayUnit === option.value}
              role="option"
              aria-selected={displayUnit === option.value}
              onclick={() => {
                displayUnit = option.value;
                closeUnitMenu();
              }}
            >
              {$t[option.labelKey]}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="sound-row">
    <label class="checkbox-field">
      <input type="checkbox" bind:checked={playSound} />
      <span class="check-box">
        {#if playSound}
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#fff"
            stroke-width="3"
          >
            <path d="M20 6L9 17l-5-5" />
          </svg>
        {/if}
      </span>
      <span>{$t.playSound}</span>
    </label>
    <button class="btn-listen" onclick={playBeep}>{$t.listenBtn}</button>
  </div>

  <!-- fullscreen toggle row -->
  <div class="fullscreen-row">
    <div class="fullscreen-left">
      <label class="checkbox-field" for="fs-checkbox">
        <input id="fs-checkbox" type="checkbox" bind:checked={fullscreen} />
        <span class="check-box">
          {#if fullscreen}
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="#fff"
              stroke-width="3"
            >
              <path d="M20 6L9 17l-5-5" />
            </svg>
          {/if}
        </span>
        <span class="fullscreen-label-text">{$t.fullscreenLabel}</span>
      </label>
      <span class="fullscreen-desc">{$t.fullscreenDesc}</span>
    </div>
    <!-- mini visual hint -->
    <div class="fs-hint" class:fs-hint--on={fullscreen}>
      {#if fullscreen}
        <div class="fs-icon fs-icon--full">
          <svg width="28" height="20" viewBox="0 0 28 20" fill="none">
            <rect
              x="1"
              y="1"
              width="26"
              height="18"
              rx="2"
              fill="var(--accent-soft)"
              stroke="var(--accent)"
              stroke-width="1.5"
            />
            <rect
              x="4"
              y="4"
              width="20"
              height="12"
              rx="1"
              fill="var(--accent)"
              opacity="0.25"
            />
          </svg>
        </div>
      {:else}
        <div class="fs-icon fs-icon--corner">
          <svg width="28" height="20" viewBox="0 0 28 20" fill="none">
            <rect
              x="1"
              y="1"
              width="26"
              height="18"
              rx="2"
              fill="none"
              stroke="var(--border)"
              stroke-width="1.5"
            />
            <rect
              x="16"
              y="11"
              width="10"
              height="7"
              rx="1.5"
              fill="var(--accent-soft)"
              stroke="var(--accent)"
              stroke-width="1.2"
            />
          </svg>
        </div>
      {/if}
    </div>
  </div>

  <div class="preview">
    <div class="preview-label">{$t.preview}</div>
    {#if fullscreen}
      <div class="preview-box preview-box--fullscreen">
        <p class="preview-text">{text || "…"}</p>
        <p class="preview-timer">{formatPreviewTimer(displaySeconds())}</p>
      </div>
    {:else}
      <div class="preview-box preview-box--corner">
        <div class="corner-card">
          <p class="corner-text">{text || "…"}</p>
          <p class="corner-timer">{formatPreviewTimer(displaySeconds())}</p>
        </div>
      </div>
    {/if}
  </div>

  <div class="actions">
    <button class="btn-save" onclick={handleSubmit}>
      {isNew ? $t.createBtn : $t.saveBtn}
    </button>
    {#if !isNew}
      <button class="btn-delete" onclick={handleDelete}>{$t.deleteBtn}</button>
    {/if}
  </div>

  {#if showDeleteConfirm}
    <div
      class="dialog-backdrop"
      role="button"
      tabindex="0"
      aria-label={$t.confirmNo}
      onclick={handleBackdropClick}
      onkeydown={handleBackdropKeydown}
    >
      <div class="dialog" role="dialog" aria-modal="true">
        <p class="dialog-title">{$t.deleteConfirm}</p>
        <p class="dialog-hint">{$t.deleteConfirmHint}</p>
        <div class="dialog-actions">
          <button class="dialog-cancel" onclick={cancelDelete}
            >{$t.confirmNo}</button
          >
          <button class="dialog-confirm" onclick={confirmDelete}
            >{$t.confirmYes}</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .text-input {
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: var(--sans);
    outline: none;
    transition: border-color 0.15s;
    resize: none;
    width: 100%;
    flex: 1;
    min-height: 80px;
  }
  .text-input:focus {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .editor {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 8px;
    height: 100%;
    position: relative;
  }

  .timing-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: nowrap;
  }
  .timing-label {
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .timing-num {
    width: 64px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--mono);
    outline: none;
    transition: border-color 0.15s;
    text-align: center;
  }
  .timing-num:focus {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .unit-select {
    position: relative;
    width: 72px;
    flex-shrink: 0;
  }

  .timing-unit {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 4px 6px 8px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--sans);
    outline: none;
    cursor: pointer;
    transition: border-color 0.15s;
    -webkit-appearance: none;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='%238b8fa3' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 6px center;
    padding-right: 20px;
    text-align: left;
  }
  .timing-unit:focus {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .unit-menu {
    position: absolute;
    top: calc(100% + 2px);
    left: 0;
    z-index: 20;
    width: 90px;
    overflow: hidden;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 2px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  .unit-option {
    display: block;
    width: 100%;
    padding: 5px 10px;
    background: var(--bg-input);
    border: 0;
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    font-family: var(--sans);
    line-height: 1.6;
    text-align: left;
    cursor: pointer;
  }

  .unit-option:hover,
  .unit-option--selected {
    background: var(--bg-card-hover);
  }

  /* ── sound row ── */
  .sound-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .checkbox-field {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-size: 14px;
  }
  .checkbox-field input {
    display: none;
  }
  .check-box {
    width: 20px;
    height: 20px;
    border: 1.5px solid var(--border);
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-input);
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .checkbox-field input:checked + .check-box {
    background: var(--accent);
    border-color: var(--accent);
  }

  .btn-listen {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px 12px;
    color: var(--text-secondary);
    font-size: 13px;
    font-family: var(--sans);
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .btn-listen:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  /* ── fullscreen row ── */
  .fullscreen-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
  }

  .fullscreen-left {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .fullscreen-label-text {
    font-size: 14px;
    color: var(--text-primary);
  }

  .fullscreen-desc {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .fs-hint {
    flex-shrink: 0;
    opacity: 0.85;
    transition: opacity 0.2s;
  }
  .fs-hint--on {
    opacity: 1;
  }

  /* ── preview ── */
  .preview {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .preview-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* fullscreen preview (original style) */
  .preview-box--fullscreen {
    background: rgba(0, 0, 0, 0.95);
    border-radius: var(--radius);
    padding: 28px 20px;
    text-align: center;
    border: 1px solid var(--border);
    height: 136px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }
  .preview-text {
    font-size: 18px;
    font-weight: 500;
    color: #fff;
    line-height: 1.6;
    margin-bottom: 12px;
  }
  .preview-timer {
    font-size: 32px;
    font-weight: 700;
    font-family: var(--mono);
    color: var(--accent);
  }

  /* corner notification preview */
  .preview-box--corner {
    background: transparent;
    border-radius: var(--radius);
    padding: 8px 0;
    display: flex;
    justify-content: flex-end;
    align-items: flex-end;
    height: 136px;
    overflow: hidden;
  }
  .corner-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 14px 18px;
    width: 220px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .corner-text {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.5;
  }
  .corner-timer {
    font-size: 20px;
    font-weight: 700;
    font-family: var(--mono);
    color: var(--accent);
  }

  /* ── actions ── */
  .actions {
    display: flex;
    gap: 10px;
    margin-top: 8px;
  }
  .btn-save {
    flex: 1;
    padding: 12px;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    font-family: var(--sans);
    transition: all 0.15s;
  }
  .btn-save:hover {
    filter: brightness(1.15);
    box-shadow: 0 0 20px var(--accent-glow);
  }
  .btn-delete {
    padding: 12px 20px;
    background: var(--danger-soft);
    color: var(--danger);
    border: 1px solid transparent;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    font-family: var(--sans);
    transition: all 0.15s;
  }
  .btn-delete:hover {
    border-color: var(--danger);
  }

  /* ── delete confirm dialog ── */
  .dialog-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    border-radius: var(--radius);
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .dialog {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 24px 24px 20px;
    width: 280px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    animation: slideUp 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .dialog-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
  }

  .dialog-hint {
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
    margin-bottom: 4px;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .dialog-cancel {
    flex: 1;
    padding: 9px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    font-family: var(--sans);
    transition: all 0.15s;
  }
  .dialog-cancel:hover {
    border-color: var(--text-secondary);
    color: var(--text-primary);
  }

  .dialog-confirm {
    flex: 1;
    padding: 9px;
    background: var(--danger-soft);
    border: 1px solid transparent;
    border-radius: 8px;
    color: var(--danger);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    font-family: var(--sans);
    transition: all 0.15s;
  }
  .dialog-confirm:hover {
    background: var(--danger);
    color: #fff;
  }
</style>
