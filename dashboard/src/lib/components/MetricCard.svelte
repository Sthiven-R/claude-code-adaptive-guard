<script lang="ts">
  import type { Snippet } from "svelte";

  /*
   * One stat card in the StatsHeader grid: an uppercase label, a big
   * value (rendered via the `value` snippet so callers can put a
   * CountUp, a formatted string, or anything else inside), and an
   * optional sub-line.
   *
   * The accent color comes from the `variant` prop, matching the
   * left-border tint that was hardcoded class-by-class in the old
   * StatsHeader.
   */

  type Variant = "default" | "block" | "deep" | "simple" | "tokens" | "window";

  let { label, value, sub, subTitle, valueSmall = false, variant = "default" }: {
    label: string;
    value: Snippet;
    sub?: string;
    subTitle?: string;
    valueSmall?: boolean;
    variant?: Variant;
  } = $props();
</script>

<div class="metric {variant}">
  <div class="label">{label}</div>
  <div class="value" class:small={valueSmall}>{@render value()}</div>
  {#if sub}
    <div class="sub" title={subTitle ?? ""}>{sub}</div>
  {/if}
</div>

<style>
  .metric {
    border-left: 2px solid var(--color-border);
    padding: 2px 0 2px 12px;
  }

  .metric.block {
    border-color: var(--color-danger);
  }
  .metric.deep {
    border-color: var(--color-success);
  }
  .metric.simple {
    border-color: var(--color-ink-faint);
  }
  .metric.tokens {
    border-color: var(--color-accent);
  }
  .metric.window {
    border-color: var(--color-accent-dim);
  }

  .label {
    font-size: var(--text-mono-micro);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--color-ink-faint);
    margin-bottom: 2px;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: var(--weight-semibold);
    color: var(--color-ink);
    letter-spacing: var(--tracking-tighter);
    font-variant-numeric: tabular-nums;
  }

  .value.small {
    font-size: var(--text-body);
  }

  /* Pass-through for the inner separator spans some callers add. */
  :global(.metric .value .sep) {
    color: var(--color-ink-faint);
    margin: 0 3px;
  }

  .sub {
    font-size: var(--text-mono-small);
    color: var(--color-ink-faint);
    font-family: var(--font-mono);
  }
</style>
