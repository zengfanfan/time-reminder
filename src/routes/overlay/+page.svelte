<script>
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  let visible = $state(false);
  let text = $state("");
  let countdown = $state(0);
  let timer = null;

  onMount(async () => {
    const win = getCurrentWebviewWindow();

    await win.listen("show-reminder", (event) => {
      const data = event.payload;
      text = data.text || "";
      countdown = data.duration || 20;
      visible = true;

      if (data.playSound) playBeep();

      if (timer) clearInterval(timer);
      timer = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
          clearInterval(timer);
          visible = false;
        }
      }, 1000);
    });
  });

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
  <div class="overlay">
    <div class="backdrop"></div>
    <div class="center-content">
      <div class="ring">
        <svg viewBox="0 0 200 200" class="ring-svg">
          <circle cx="100" cy="100" r="90" class="ring-bg" />
          <circle cx="100" cy="100" r="90" class="ring-progress" />
        </svg>
        <span class="timer">{formatCountdown(countdown)}</span>
      </div>
      <p class="message">{text}</p>
      <p class="hint">倒计时结束后自动关闭</p>
    </div>
  </div>
{/if}

<style>
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
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background: rgba(8, 10, 18, 0.92);
    backdrop-filter: blur(30px);
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
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .timer {
    font-family: "JetBrains Mono", monospace;
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
</style>
