<script lang="ts">
  import { t } from "../i18n";

  /*
   * Live indicator. When active, a solid dot pulses while a concentric
   * ring expands and fades — the same visual idiom as a "recording"
   * light on broadcast equipment. The asymmetric tempo (fast inner
   * pulse, slow outer ring) reads as "watching" rather than "loading."
   *
   * When paused, the dot drops to ink-faint and both animations stop.
   */

  let { active }: { active: boolean } = $props();
</script>

<span
  class="live"
  class:active
  title={active ? $t.live.on_tooltip : $t.live.off_tooltip}
>
  <span class="dot-wrap">
    <span class="dot" aria-hidden="true"></span>
    {#if active}
      <span class="ring" aria-hidden="true"></span>
    {/if}
  </span>
  <span class="text">{active ? $t.live.on : $t.live.off}</span>
</span>

<style>
  .live {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 2px 10px 2px 8px;
    border-radius: var(--radius-pill);
    font-family: var(--font-mono);
    font-size: var(--text-mono-micro);
    letter-spacing: var(--tracking-widest);
    color: var(--color-ink-faint);
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    transition: color var(--duration-base) var(--ease-standard),
                border-color var(--duration-base) var(--ease-standard);
  }

  .dot-wrap {
    position: relative;
    width: 8px;
    height: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .dot {
    position: relative;
    z-index: 2;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--color-ink-faint);
    transition: background var(--duration-base) var(--ease-standard),
                box-shadow var(--duration-base) var(--ease-standard);
  }

  /* The expanding ring: starts at the dot's footprint, grows ~3x,
   * fades to zero. Re-spawns every cycle. */
  .ring {
    position: absolute;
    z-index: 1;
    inset: 0;
    border-radius: 50%;
    border: 1.5px solid var(--color-success);
    animation: ring-expand 1.6s var(--ease-out-expo) infinite;
    pointer-events: none;
  }

  .live.active {
    color: var(--color-success);
    border-color: var(--color-success-soft);
  }
  .live.active .dot {
    background: var(--color-success);
    box-shadow: 0 0 6px var(--color-success);
    animation: dot-pulse 1.6s var(--ease-standard) infinite;
  }

  @keyframes dot-pulse {
    0%, 100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(0.78);
      opacity: 0.65;
    }
  }

  @keyframes ring-expand {
    0% {
      transform: scale(0.85);
      opacity: 0.7;
    }
    100% {
      transform: scale(3.2);
      opacity: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .ring,
    .live.active .dot {
      animation: none;
    }
  }
</style>
