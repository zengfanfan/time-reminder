<script>
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { t, initLocale } from "$lib/i18n.js";

  // Data filled after backend fetch
  let reminderId = $state("");
  let windowLabel = $state("");
  let text = $state("");
  let countdown = $state(0);
  let totalDuration = $state(0);
  let fullscreen = $state(false);
  let soundVolume = $state(60);
  let visible = $state(false);

  let timer = null;
  let win = null;

  // Corner slide-down animation when repositioned
  let repositioning = $state(false);

  onMount(async () => {
    initLocale();
    win = getCurrentWebviewWindow();
    windowLabel = win.label;

    // Fetch our own payload from the backend
    const data = await invoke("get_overlay_data", { label: windowLabel });
    if (!data) {
      // Shouldn't happen, close self
      win.close();
      return;
    }

    reminderId = data.id ?? "";
    text = data.text ?? "";
    countdown = data.duration ?? 20;
    totalDuration = countdown;
    fullscreen = data.fullscreen ?? false;
    soundVolume = data.volume ?? 60;
    visible = true;

    // For fullscreen: wait for Svelte to flush DOM updates so the WebView
    // has actually painted before we reveal the window — kills the white flash.
    // Corner windows are small/transparent so they don't need this treatment.
    if (fullscreen) {
      await tick();
      await invoke("show_overlay", { label: windowLabel, fullscreen: true });
    }

    if (data.playSound) playBeep();

    // Start countdown
    timer = setInterval(() => {
      countdown--;
      if (countdown <= 0) dismiss();
    }, 1000);

    // Listen for re-layout repositioning (corner stack only)
    if (!fullscreen) {
      await win.listen("reposition", () => {
        // Trigger a brief CSS transition on the card
        repositioning = true;
        setTimeout(() => (repositioning = false), 350);
      });
    }
  });

  function dismiss() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    visible = false;
    invoke("dismiss_overlay", {
      label: windowLabel,
      reminderId,
      fullscreen,
    }).catch(() => {});
  }

  function playBeep() {
    try {
      const vol = (soundVolume ?? 60) / 100;
      const ctx = new AudioContext();
      const makeBeep = (freq, delay) => {
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
      makeBeep(660, 0);
      makeBeep(880, 300);
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

      <button class="btn-dismiss" onclick={dismiss}>
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
    <div class="corner-wrapper" class:repositioning>
      <div class="corner-card" role="alertdialog" aria-live="assertive">
        <!-- draining background — sits behind everything -->
        {#key totalDuration}
          <div
            class="corner-progress-bg"
            style="animation-duration: {totalDuration}s"
          ></div>
        {/key}

        <span class="corner-dot"></span>
        <span class="corner-title">{text}</span>
        <span class="corner-timer">{formatCountdown(countdown)}</span>
        <button class="corner-close" onclick={dismiss} aria-label={$t.dismiss}>
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
    </div>
  {/if}
{/if}

<style>
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
    background: rgba(8, 10, 18, 0.95);
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
    /* Rust positions the window itself, so the card fills the whole window */
    inset: 0;
    display: flex;
    align-items: stretch;
    animation: cornerSlideIn 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  }
  /* When Rust repositions the window, add a smooth CSS transition */
  .corner-wrapper.repositioning {
    transition: top 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes cornerSlideIn {
    from {
      opacity: 0;
      transform: translateX(24px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateX(0) scale(1);
    }
  }

  .corner-card {
    /* single row, vertically centered */
    position: relative;
    flex: 1;
    background: #1a1d2e;
    overflow: hidden;
    border-radius: 22px;
    padding: 0 10px 0 12px;
    border: 1px solid #0000;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* progress background: full-width accent color that drains left→right */
  .corner-progress-bg {
    position: absolute;
    inset: 0;
    background: rgba(78, 159, 255, 0.3);
    transform-origin: left;
    animation: progressDrain linear forwards;
    pointer-events: none;
    border-radius: 0;
  }
  @keyframes progressDrain {
    from {
      transform: scaleX(1);
    }
    to {
      transform: scaleX(0);
    }
  }

  .corner-dot {
    position: relative; /* above progress bg */
    flex-shrink: 0;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #6d6;
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
    position: relative;
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: #e8eaf0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .corner-timer {
    position: relative;
    flex-shrink: 0;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--mono);
    color: #6bf;
    min-width: 28px;
    text-align: right;
  }

  .corner-close {
    position: relative;
    flex-shrink: 0;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }
  .corner-close:hover {
    color: #ff4e6a;
    background: rgba(255, 78, 106, 0.15);
  }
</style>
