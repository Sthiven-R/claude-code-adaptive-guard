<script lang="ts">
  /*
   * Adaptive Guard logo.
   *
   * Mark: a 3/4 arc (the "guard" — open at the top-right, signaling
   * adaptiveness, not a closed shield) with a heartbeat trace
   * crossing the center (the "monitoring" — every Claude Code
   * decision pulses through). Asymmetric on purpose: a perfectly
   * symmetric mark reads as static / decorative; the off-center
   * pulse reads as live.
   *
   * Colors come from the design tokens, never hardcoded — flipping
   * the theme should re-tint the logo without touching this file.
   *
   * Variants:
   *   - "symbol"   →  just the mark.    Use as favicon / app icon.
   *   - "wordmark" →  just the wordmark. Rarely needed alone.
   *   - "full"     →  mark + wordmark.   Use in headers, README hero.
   */

  type Variant = "symbol" | "wordmark" | "full";

  let { variant = "full", size = 24, title = "Adaptive Guard" }: {
    variant?: Variant;
    size?: number;
    title?: string;
  } = $props();

  const wordmarkHeight = $derived(Math.round(size * 0.85));
</script>

{#if variant === "symbol" || variant === "full"}
  <svg
    class="ag-symbol"
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    role="img"
    aria-label={title}
  >
    <title>{title}</title>
    <!--
      Outer arc: 3/4 of a circle, opening at the top-right quadrant.
      The opening is what makes this read as "adaptive guard"
      instead of a closed shield. radius=11, centered at (16, 16).
      Path math: starts at angle 30°, sweeps 270° clockwise.
    -->
    <path
      d="M 25.53 11.5 A 11 11 0 1 0 21.5 25.53"
      stroke="var(--color-accent)"
      stroke-width="2.5"
      stroke-linecap="round"
    />
    <!--
      Inner pulse: 5-segment heartbeat trace. Sits below center so
      the visual balance is asymmetric — the eye lands on the pulse
      first, then traces the arc that surrounds it.
    -->
    <path
      d="M 7 17 L 11.5 17 L 13.5 13 L 17 21 L 19 17 L 25 17"
      stroke="var(--color-accent)"
      stroke-width="2.25"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
{/if}

{#if variant === "wordmark" || variant === "full"}
  <span class="ag-wordmark" style="height: {wordmarkHeight}px; line-height: {wordmarkHeight}px;">
    adaptive<span class="ag-mid-dot">·</span>guard
  </span>
{/if}

<style>
  :global(.ag-symbol) {
    display: inline-block;
    flex-shrink: 0;
  }

  .ag-wordmark {
    /* Tokens are loaded synchronously by main.ts before any component
     * mounts (see main.ts ordering); inline fallbacks are theatre that
     * misrepresent the light-theme value. Trust the token. */
    font-family: var(--font-mono);
    font-weight: var(--weight-semibold);
    font-size: 0.95em;
    letter-spacing: var(--tracking-tight);
    color: var(--color-ink);
    display: inline-block;
    vertical-align: middle;
    margin-left: 0.4em;
  }

  .ag-mid-dot {
    color: var(--color-accent);
    margin: 0 0.05em;
    font-weight: var(--weight-bold);
  }
</style>
