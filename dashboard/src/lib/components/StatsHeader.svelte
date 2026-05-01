<script lang="ts">
  import type { TelemetryStats } from "../types";
  import { formatRelative } from "../time";
  import { t } from "../i18n";
  import Logo from "./Logo.svelte";
  import CountUp from "./CountUp.svelte";
  import Skeleton from "./Skeleton.svelte";

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
  <div class="top-row">
    <div class="brand">
      <Logo variant="full" size={22} />
    </div>
    <div class="meta">
      <span class="profile">{$t.stats.profile} <strong>{profile}</strong></span>
      <span class="version">{version ? `v${version}` : ""}</span>
    </div>
  </div>

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
      <div class="metric">
        <div class="label">{$t.stats.total}</div>
        <div class="value"><CountUp value={stats.total} /></div>
      </div>

      <div class="metric block">
        <div class="label">{$t.stats.blocks}</div>
        <div class="value"><CountUp value={stats.block_count} /></div>
        <div class="sub">{pct(stats.block_ratio)}</div>
      </div>

      <div class="metric deep">
        <div class="label">{$t.stats.deep_allowed}</div>
        <div class="value"><CountUp value={stats.allow_deep_count} /></div>
        <div class="sub">{pct(stats.deep_ratio)}</div>
      </div>

      <div class="metric simple">
        <div class="label">{$t.stats.simple_skipped}</div>
        <div class="value"><CountUp value={stats.allow_simple_count} /></div>
        <div class="sub">{pct(stats.simple_ratio)}</div>
      </div>

      <div class="metric tokens">
        <div class="label">{$t.stats.tokens_in_out}</div>
        <div class="value">
          <CountUp value={stats.approx_tokens_from_chars_in} />
          <span class="sep">/</span>
          <CountUp value={stats.approx_tokens_from_chars_out} />
        </div>
        <div class="sub" title={$t.stats.chars_estimate_hint}>
          {$t.stats.chars_estimate}
        </div>
      </div>

      <div class="metric window">
        <div class="label">{$t.stats.since}</div>
        <div class="value small">
          {stats.first_ts ? formatRelative(stats.first_ts, $t.time, now) : "—"}
        </div>
        <div class="sub">
          {$t.stats.last} {stats.last_ts ? formatRelative(stats.last_ts, $t.time, now) : "—"}
        </div>
      </div>
    </div>

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

  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .brand {
    display: inline-flex;
    align-items: center;
  }

  .meta {
    display: flex;
    gap: var(--space-4);
    color: var(--color-ink-dim);
    font-size: var(--text-small);
    font-family: var(--font-mono);
  }

  .meta strong {
    color: var(--color-ink);
    font-weight: var(--weight-semibold);
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-3);
  }

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

  .skeleton-metric {
    border-color: var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .skeleton-metric .sk-value {
    margin: 2px 0;
  }

  .metric .label {
    font-size: var(--text-mono-micro);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--color-ink-faint);
    margin-bottom: 2px;
  }
  .metric .value {
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: var(--weight-semibold);
    color: var(--color-ink);
    letter-spacing: var(--tracking-tighter);
    font-variant-numeric: tabular-nums;
  }
  .metric .value.small {
    font-size: var(--text-body);
  }
  .metric .value .sep {
    color: var(--color-ink-faint);
    margin: 0 3px;
  }
  .metric .sub {
    font-size: var(--text-mono-small);
    color: var(--color-ink-faint);
    font-family: var(--font-mono);
  }

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

  .empty {
    color: var(--color-ink-faint);
    font-size: var(--text-small);
    padding: var(--space-2) 0;
  }
</style>
