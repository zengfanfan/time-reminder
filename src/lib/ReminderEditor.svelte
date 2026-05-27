<script>
  import { tick } from "svelte";

  let { config, isNew, onSave, onDelete, name, onNameChange, triggerSave } =
    $props();

  let text = $state(config.text);
  let playSound = $state(config.play_sound);

  // Interval: value + unit
  let intervalValue = $state(deriveValue(config.interval_secs));
  let intervalUnit = $state(deriveUnit(config.interval_secs));

  // Display duration: value + unit
  let displayValue = $state(deriveValue(config.display_secs));
  let displayUnit = $state(deriveUnit(config.display_secs));

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

  function unitLabel(unit) {
    if (unit === "hours") return "小时";
    if (unit === "minutes") return "分钟";
    return "秒";
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
  let prevTriggerSave = triggerSave;
  $effect(() => {
    if (triggerSave > prevTriggerSave) {
      prevTriggerSave = triggerSave;
      handleSubmit();
    }
  });

  function handleSubmit() {
    onSave({
      ...config,
      name,
      text,
      interval_secs: toSeconds(intervalValue, intervalUnit),
      display_secs: toSeconds(displayValue, displayUnit),
      play_sound: playSound,
      enabled: config.enabled,
    });
  }

  function handleDelete() {
    if (confirm("确认删除这个提醒？")) {
      onDelete(config.id);
    }
  }

  function playBeep() {
    try {
      const ctx = new AudioContext();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.frequency.value = 660;
      osc.type = "sine";
      gain.gain.setValueAtTime(0.3, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.5);
      osc.start(ctx.currentTime);
      osc.stop(ctx.currentTime + 0.5);
      setTimeout(() => {
        const osc2 = ctx.createOscillator();
        const gain2 = ctx.createGain();
        osc2.connect(gain2);
        gain2.connect(ctx.destination);
        osc2.frequency.value = 880;
        osc2.type = "sine";
        gain2.gain.setValueAtTime(0.3, ctx.currentTime);
        gain2.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.5);
        osc2.start(ctx.currentTime);
        osc2.stop(ctx.currentTime + 0.5);
      }, 300);
    } catch (e) {
      console.warn("Audio playback failed:", e);
    }
  }
</script>

<div class="editor">
  <textarea
    class="text-input"
    bind:value={text}
    placeholder="遮挡屏幕时显示的文字…"
  ></textarea>

  <div class="timing-row">
    <span class="timing-label">每</span>
    <input
      class="timing-num"
      type="number"
      bind:value={intervalValue}
      min="1"
    />
    <select class="timing-unit" bind:value={intervalUnit}>
      <option value="seconds">秒</option>
      <option value="minutes">分钟</option>
      <option value="hours">小时</option>
    </select>
    <span class="timing-label">提醒一次，显示</span>
    <input class="timing-num" type="number" bind:value={displayValue} min="1" />
    <select class="timing-unit" bind:value={displayUnit}>
      <option value="seconds">秒</option>
      <option value="minutes">分钟</option>
      <option value="hours">小时</option>
    </select>
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
      <span>提醒时播放提示音</span>
    </label>
    <button class="btn-listen" onclick={playBeep} title="试听提示音">
      🔊 试听
    </button>
  </div>

  <div class="preview">
    <div class="preview-label">预览</div>
    <div class="preview-box">
      <p class="preview-text">{text || "…"}</p>
      <p class="preview-timer">{formatPreviewTimer(displaySeconds())}</p>
    </div>
  </div>

  <div class="actions">
    <button class="btn-save" onclick={handleSubmit}>
      {isNew ? "创建提醒" : "保存修改"}
    </button>
    {#if !isNew}
      <button class="btn-delete" onclick={handleDelete}>删除</button>
    {/if}
  </div>
</div>

<style>
  .text-input {
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: "Noto Sans SC", sans-serif;
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

  .timing-unit {
    width: 72px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 4px 6px 8px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: "Noto Sans SC", sans-serif;
    outline: none;
    cursor: pointer;
    transition: border-color 0.15s;
    -webkit-appearance: none;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='%238b8fa3' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 6px center;
    padding-right: 20px;
  }

  .timing-unit:focus {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

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
    font-family: "Noto Sans SC", sans-serif;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn-listen:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

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

  .preview-box {
    background: rgba(0, 0, 0, 0.85);
    border-radius: var(--radius);
    padding: 28px 20px;
    text-align: center;
    border: 1px solid var(--border);
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
    font-family: "Noto Sans SC", sans-serif;
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
    font-family: "Noto Sans SC", sans-serif;
    transition: all 0.15s;
  }

  .btn-delete:hover {
    border-color: var(--danger);
  }
</style>
