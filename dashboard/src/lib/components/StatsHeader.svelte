<script lang="ts">
  import type { TelemetryStats } from "../types";
  import { formatRelative } from "../time";
  import { t } from "../i18n";
  import BrandRow from "./BrandRow.svelte";
  import MetricCard from "./MetricCard.svelte";
  import ScoreLine from "./ScoreLine.svelte";
  import CountUp from "./CountUp.svelte";
  import Skeleton from "./Skeleton.svelte";

  /*
   * Layout shell: brand row, the 6-card metric grid (or a 6-card
   * skeleton, or an empty-state line), and the score-line strip.
   * Each of those is now its own component — see BrandRow,
   * MetricCard, and ScoreLine.
   */

  let { stats, profile, version, now }: {
    stats: TelemetryStats | null;
    profile: string;
    version: string;
    now: Date;
  } = $props();

  function pct(n: number): string {
    return `${(n * 100).toFixed(1)}%`;
  }
</script>

<section class="stats-header">
  <BrandRow {profile} {version} />

  {#if !stats}
    <!--
      Loading skeleton — six metric placeholders that mirror the live
      grid below. Reads as "the panel is going to look like X" instead
      of a bare "Loading…" string.
    -->
    <div class="metrics" aria-busy="true" aria-label={$t.stats.loading}>
      {#each Array(6) as _, i (i)}
        <div class="metric skeleton-metric">
          <Skeleton width="56px" height="10px" />
          <div class="sk-value"><Skeleton width="80px" height="22px" /></div>
          <Skeleton width="40px" height="10px" />
        </div>
      {/each}
    </div>
  {:else if stats.total === 0}
    <div class="empty">
      {$t.stats.no_decisions}
    </div>
  {:else}
    <div class="metrics">
      <MetricCard label={$t.stats.total}>
        {#snippet value()}<CountUp value={stats.total} />{/snippet}
      </MetricCard>

      <MetricCard label={$t.stats.blocks} variant="block" sub={pct(stats.block_ratio)}>
        {#snippet value()}<CountUp value={stats.block_count} />{/snippet}
      </MetricCard>

      <MetricCard label={$t.stats.deep_allowed} variant="deep" sub={pct(stats.deep_ratio)}>
        {#snippet value()}<CountUp value={stats.allow_deep_count} />{/snippet}
      </MetricCard>

      <MetricCard label={$t.stats.simple_skipped} variant="simple" sub={pct(stats.simple_ratio)}>
        {#snippet value()}<CountUp value={stats.allow_simple_count} />{/snippet}
      </MetricCard>

      <MetricCard
        label={$t.stats.tokens_in_out}
        variant="tokens"
        sub={$t.stats.chars_estimate}
        subTitle={$t.stats.chars_estimate_hint}
      >
        {#snippet value()}
          <CountUp value={stats.approx_tokens_from_chars_in} />
          <span class="sep">/</span>
          <CountUp value={stats.approx_tokens_from_chars_out} />
        {/snippet}
      </MetricCard>

      <MetricCard
        label={$t.stats.since}
        variant="window"
        valueSmall
        sub={`${$t.stats.last} ${stats.last_ts ? formatRelative(stats.last_ts, $t.time, now) : "—"}`}
      >
        {#snippet value()}
          {stats.first_ts ? formatRelative(stats.first_ts, $t.time, now) : "—"}
        {/snippet}
      </MetricCard>
    </div>

    <ScoreLine {stats} />
  {/if}
</section>

<style>
  .stats-header {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 18px 22px 16px;
    margin-bottom: var(--space-5);
    box-shadow: var(--elevation-1);
    transition: box-shadow var(--duration-base) var(--ease-standard);
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-3);
  }

  /* Skeleton-metric placeholder uses the same left-border treatment as
   * MetricCard's default variant (kept inline because the skeleton is
   * a visual rhythm, not a reusable card). */
  .metric.skeleton-metric {
    border-left: 2px solid var(--color-border);
    padding: 2px 0 2px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .skeleton-metric .sk-value {
    margin: 2px 0;
  }

  .empty {
    color: var(--color-ink-faint);
    font-size: var(--text-small);
    padding: var(--space-2) 0;
  }
</style>
