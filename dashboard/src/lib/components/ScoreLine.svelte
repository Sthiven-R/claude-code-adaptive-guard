<script lang="ts">
  import type { TelemetryStats } from "../types";
  import { t } from "../i18n";

  /*
   * Bottom strip of StatsHeader: average complexity / depth / missing
   * for the BLOCK and DEEP buckets. Renders nothing if neither bucket
   * has any records — extracted as its own component so the parent
   * does not have to reason about the conditional emptiness.
   */

  let { stats }: { stats: TelemetryStats } = $props();
</script>

{#if stats.block_count > 0 || stats.allow_deep_count > 0}
  <div class="score-line">
    {#if stats.block_count > 0}
      <span>
        <span class="tag block-tag">{$t.stats.block_tag}</span>
        {$t.stats.avg_complexity}
        <strong>{stats.avg_complexity_block.toFixed(1)}</strong>
        · {$t.stats.depth}
        <strong>{stats.avg_depth_block.toFixed(1)}</strong>
        · {$t.stats.missing}
        <strong>{stats.avg_missing_block.toFixed(1)}</strong>
      </span>
    {/if}
    {#if stats.allow_deep_count > 0}
      <span>
        <span class="tag deep-tag">{$t.stats.deep_tag}</span>
        {$t.stats.avg_complexity}
        <strong>{stats.avg_complexity_deep.toFixed(1)}</strong>
        · {$t.stats.depth}
        <strong>{stats.avg_depth_deep.toFixed(1)}</strong>
      </span>
    {/if}
  </div>
{/if}

<style>
  .score-line {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-5);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-border);
    font-size: var(--text-small);
    color: var(--color-ink-dim);
    font-family: var(--font-mono);
  }

  .score-line strong {
    color: var(--color-ink);
    font-weight: var(--weight-semibold);
  }

  .tag {
    display: inline-block;
    padding: 1px 8px;
    border-radius: var(--radius-xs);
    font-size: var(--text-mono-micro);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
    margin-right: 4px;
  }

  .tag.block-tag {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .tag.deep-tag {
    background: var(--color-success-soft);
    color: var(--color-success);
  }
</style>
