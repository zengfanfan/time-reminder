<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { t, initLocale } from "$lib/i18n.js";

  let visible = $state(false);
  let text = $state("");
  let countdown = $state(0);
  let totalDuration = $state(0); // fixed initial duration for progress bar
  let fullscreen = $state(false);
  let currentId = $state("");
  let timer = null;
  let win = null;

  onMount(async () => {
    initLocale();
    win = getCurrentWebviewWindow();

    await win.listen("show-reminder", (event) => {
      const data = event.payload;
      currentId = data.id ?? "";
      text = data.text || "";
      countdown = data.duration || 20;
      totalDuration = countdown;
      fullscreen = data.fullscreen ?? false;
      visible = true;

      if (data.playSound) playBeep();

      if (timer) clearInterval(timer);
      timer = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
          dismiss();
        }
      }, 1000);
    });
  });

  function dismiss() {
    if (timer) clearInterval(timer);
    timer = null;
    visible = false;
    // Tell the backend the break is over — interval restarts from now.
    if (currentId)
      invoke("dismiss_reminder", { id: currentId }).catch(() => {});
    if (win) win.hide();
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

  function formatCountdown(secs) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    if (m > 0) return `${m}:${s.toString().padStart(2, "0")}`;
    return `${s}`;
  }
</script>

{#if visible}
  {#if fullscreen}
    <!-- ── Fullscreen overlay ── -->
    <div class="overlay">
      <div class="backdrop"></div>

      <button class="btn-dismiss" onclick={dismiss} title={$t.dismiss}>
        <svg
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
        <span>{$t.dismiss}</span>
      </button>

      <div class="center-content">
        <div class="ring">
          <svg viewBox="0 0 200 200" class="ring-svg">
            <circle cx="100" cy="100" r="90" class="ring-bg" />
            <circle cx="100" cy="100" r="90" class="ring-progress" />
          </svg>
          <span class="timer">{formatCountdown(countdown)}</span>
        </div>
        <p class="message">{text}</p>
        <p class="hint">{$t.dismissHint}</p>
      </div>
    </div>
  {:else}
    <!-- ── Corner notification ── -->
    <div class="corner-wrapper">
      <div class="corner-card" role="alertdialog" aria-live="assertive">
        <div class="corner-header">
          <span class="corner-dot"></span>
          <span class="corner-title">{text}</span>
          <button class="corner-close" onclick={dismiss} title={$t.dismiss}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
            >
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="corner-footer">
          <div class="corner-progress-bar">
            {#key totalDuration}
              <div
                class="corner-progress-fill"
                style="animation-duration: {totalDuration}s"
              ></div>
            {/key}
          </div>
          <span class="corner-timer">{formatCountdown(countdown)}</span>
        </div>
      </div>
    </div>
  {/if}
{/if}

<style>
  /* Make the window itself fully transparent — app.css sets body background which we must override */
  :global(html),
  :global(body) {
    background: transparent !important;
  }

  /* ════ Fullscreen ════ */
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 99999;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadeIn 0.4s ease-out;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background: rgba(8, 10, 18, 0.92);
    backdrop-filter: blur(30px);
  }

  .btn-dismiss {
    position: absolute;
    top: 32px;
    right: 40px;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    padding: 10px 18px 10px 14px;
    color: rgba(255, 255, 255, 0.5);
    font-size: 14px;
    font-family: var(--sans);
    cursor: pointer;
    transition: all 0.2s;
    backdrop-filter: blur(8px);
  }
  .btn-dismiss:hover {
    background: rgba(255, 78, 106, 0.15);
    border-color: rgba(255, 78, 106, 0.4);
    color: #ff4e6a;
  }

  .center-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    animation: slideUp 0.5s cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(30px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .ring {
    position: relative;
    width: 180px;
    height: 180px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .ring-svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  .ring-bg {
    fill: none;
    stroke: rgba(78, 123, 255, 0.1);
    stroke-width: 4;
  }
  .ring-progress {
    fill: none;
    stroke: #4e7bff;
    stroke-width: 4;
    stroke-linecap: round;
    stroke-dasharray: 565.48;
    stroke-dashoffset: 0;
    filter: drop-shadow(0 0 12px rgba(78, 123, 255, 0.4));
    animation: ringPulse 2s ease-in-out infinite;
  }
  @keyframes ringPulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  .timer {
    font-family: var(--sans);
    font-size: 52px;
    font-weight: 700;
    color: #fff;
    letter-spacing: -0.02em;
    text-shadow: 0 0 40px rgba(78, 123, 255, 0.3);
  }
  .message {
    font-size: 26px;
    font-weight: 500;
    color: #e8eaf0;
    text-align: center;
    max-width: 600px;
    line-height: 1.6;
  }
  .hint {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.3);
    letter-spacing: 0.05em;
  }

  /* ════ Corner notification ════ */
  .corner-wrapper {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 99999;
    animation: cornerSlideIn 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes cornerSlideIn {
    from {
      opacity: 0;
      transform: translateX(24px) translateY(8px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateX(0) translateY(0) scale(1);
    }
  }

  .corner-card {
    width: 300px;
    background: #1a1d2e;
    border-radius: 14px;
    padding: 14px 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .corner-header {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }
  .corner-dot {
    flex-shrink: 0;
    margin-top: 3px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4e7bff;
    box-shadow: 0 0 6px rgba(78, 123, 255, 0.8);
    animation: dotPulse 2s ease-in-out infinite;
  }
  @keyframes dotPulse {
    0%,
    100% {
      opacity: 1;
      box-shadow: 0 0 6px rgba(78, 123, 255, 0.8);
    }
    50% {
      opacity: 0.6;
      box-shadow: 0 0 10px rgba(78, 123, 255, 0.4);
    }
  }
  .corner-title {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: #e8eaf0;
    line-height: 1.5;
  }
  .corner-close {
    flex-shrink: 0;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.3);
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }
  .corner-close:hover {
    color: #ff4e6a;
    background: rgba(255, 78, 106, 0.1);
  }
  .corner-footer {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .corner-progress-bar {
    flex: 1;
    height: 3px;
    background: rgba(78, 123, 255, 0.15);
    border-radius: 2px;
    overflow: hidden;
  }
  .corner-progress-fill {
    height: 100%;
    width: 100%;
    background: linear-gradient(90deg, #4e7bff, #7b9fff);
    border-radius: 2px;
    transform-origin: left;
    animation: progressDrain linear forwards;
  }
  @keyframes progressDrain {
    from {
      transform: scaleX(1);
    }
    to {
      transform: scaleX(0);
    }
  }
  .corner-timer {
    flex-shrink: 0;
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 600;
    color: #4e7bff;
    min-width: 36px;
    text-align: right;
  }
</style>
