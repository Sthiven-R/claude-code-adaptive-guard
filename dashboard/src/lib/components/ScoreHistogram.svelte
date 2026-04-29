<script lang="ts">
  import type { HistogramBucket } from "../types";
  import { t } from "../i18n";

  let { buckets, title, threshold }: {
    buckets: HistogramBucket[];
    title: string;
    threshold: number | null;
  } = $props();

  // Build a full 0..90 bucket array so the chart always has the same
  // x-axis even when some buckets are empty.
  const allBuckets = $derived.by(() => {
    const byLo = new Map<number, number>();
    for (const b of buckets) byLo.set(b.bucket_lo, b.count);
    const filled: HistogramBucket[] = [];
    for (let lo = 0; lo < 100; lo += 10) {
      filled.push({
        bucket_lo: lo,
        bucket_hi: lo + 9,
        count: byLo.get(lo) ?? 0,
      });
    }
    return filled;
  });

  const maxCount = $derived(
    allBuckets.reduce((acc, b) => Math.max(acc, b.count), 0)
  );
  const total = $derived(allBuckets.reduce((acc, b) => acc + b.count, 0));

  function barHeight(count: number): string {
    if (maxCount === 0) return "2px";
    const h = 2 + (count / maxCount) * 100;
    return `${h}px`;
  }

  function barColor(lo: number): string {
    if (threshold === null) return "var(--accent)";
    return lo + 10 <= threshold ? "var(--ink-faint)" : "var(--accent)";
  }
</script>

<div class="histogram">
  <div class="head">
    <h3>{title}</h3>
    <span class="total">{total} {$t.histogram.records}</span>
  </div>

  {#if total === 0}
    <div class="empty">{$t.histogram.no_data}</div>
  {:else}
    <div class="bars">
      {#each allBuckets as b}
        <div class="col">
          <div class="bar-wrap">
            <div
              class="bar"
              style="height: {barHeight(b.count)}; background: {barColor(b.bucket_lo)}"
              title="{b.bucket_lo}-{b.bucket_hi}: {b.count}"
            ></div>
          </div>
          <div class="label" class:dim={b.count === 0}>{b.bucket_lo}</div>
        </div>
      {/each}
    </div>
    {#if threshold !== null}
      <div class="legend">
        <span class="swatch below"></span> {$t.histogram.below_threshold} ({threshold})
        <span class="swatch above"></span> {$t.histogram.at_or_above}
      </div>
    {/if}
  {/if}
</div>

<style>
  .histogram {
    background: var(--bg-soft);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 14px 18px 12px;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 12px;
  }

  h3 {
    margin: 0;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ink-dim);
    font-weight: 600;
  }

  .total {
    color: var(--ink-faint);
    font-size: 11px;
    font-family: var(--mono);
  }

  .bars {
    display: grid;
    grid-template-columns: repeat(10, 1fr);
    gap: 4px;
    height: 120px;
    align-items: end;
  }

  .col {
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
  }

  .bar-wrap {
    flex: 1;
    width: 100%;
    display: flex;
    align-items: flex-end;
    justify-content: center;
  }

  .bar {
    width: 90%;
    border-radius: 3px 3px 0 0;
    transition: height 0.2s ease;
  }

  .label {
    margin-top: 4px;
    font-family: var(--mono);
    font-size: 10px;
    color: var(--ink-dim);
  }
  .label.dim {
    color: var(--ink-faint);
  }

  .legend {
    margin-top: 8px;
    font-size: 11px;
    color: var(--ink-faint);
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .swatch {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 2px;
    margin: 0 2px 0 4px;
  }
  .swatch.below {
    background: var(--ink-faint);
  }
  .swatch.above {
    background: var(--accent);
  }

  .empty {
    color: var(--ink-faint);
    font-size: 12px;
    text-align: center;
    padding: 20px 0;
  }
</style>
