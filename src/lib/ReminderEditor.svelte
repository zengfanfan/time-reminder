<script>
  let { config, onSave, onDelete } = $props();

  let name = $state(config.name);
  let text = $state(config.text);
  let intervalMin = $state(Math.floor(config.interval_secs / 60));
  let displaySec = $state(config.display_secs);
  let playSound = $state(config.play_sound);
  let isNew = config.name === "新提醒";

  function handleSubmit() {
    onSave({
      ...config,
      name,
      text,
      interval_secs: intervalMin * 60,
      display_secs: displaySec,
      play_sound: playSound,
      enabled: config.enabled,
    });
  }

  function handleDelete() {
    if (confirm("确认删除这个提醒？")) {
      onDelete(config.id);
    }
  }
</script>

<div class="editor">
  <div class="field">
    <label for="name">提醒名称</label>
    <input id="name" type="text" bind:value={name} placeholder="例：护眼提醒" />
  </div>

  <div class="field">
    <label for="text">显示文本</label>
    <textarea id="text" bind:value={text} rows="3" placeholder="遮挡屏幕时显示的文字…"></textarea>
  </div>

  <div class="field-row">
    <div class="field">
      <label for="interval">提醒间隔（分钟）</label>
      <input id="interval" type="number" bind:value={intervalMin} min="1" max="480" />
    </div>
    <div class="field">
      <label for="duration">显示时长（秒）</label>
      <input id="duration" type="number" bind:value={displaySec} min="5" max="600" />
    </div>
  </div>

  <label class="checkbox-field">
    <input type="checkbox" bind:checked={playSound} />
    <span class="check-box">
      {#if playSound}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="3">
          <path d="M20 6L9 17l-5-5" />
        </svg>
      {/if}
    </span>
    <span>提醒时播放提示音 🔔</span>
  </label>

  <div class="preview">
    <div class="preview-label">预览</div>
    <div class="preview-box">
      <p class="preview-text">{text || "…"}</p>
      <p class="preview-timer">{displaySec}</p>
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
  .editor {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 8px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .field label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .field input,
  .field textarea {
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
  }

  .field input:focus,
  .field textarea:focus {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .field input[type="number"] {
    font-family: var(--mono);
  }

  .field-row {
    display: flex;
    gap: 12px;
  }

  .checkbox-field {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-size: 14px;
    padding: 10px 0;
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
