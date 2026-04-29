<script lang="ts">
  import type { TelemetryStats } from "../types";
  import { formatRelative } from "../time";
  import { t } from "../i18n";

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
      <span class="dot"></span>
      adaptive-guard
    </div>
    <div class="meta">
      <span class="profile">{$t.stats.profile} <strong>{profile}</strong></span>
      <span class="version">{version ? `v${version}` : ""}</span>
    </div>
  </div>

  {#if stats}
    {#if stats.total === 0}
      <div class="empty">
        {$t.stats.no_decisions}
      </div>
    {:else}
      <div class="metrics">
        <div class="metric">
          <div class="label">{$t.stats.total}</div>
          <div class="value">{stats.total.toLocaleString()}</div>
        </div>

        <div class="metric block">
          <div class="label">{$t.stats.blocks}</div>
          <div class="value">{stats.block_count}</div>
          <div class="sub">{pct(stats.block_ratio)}</div>
        </div>

        <div class="metric deep">
          <div class="label">{$t.stats.deep_allowed}</div>
          <div class="value">{stats.allow_deep_count}</div>
          <div class="sub">{pct(stats.deep_ratio)}</div>
        </div>

        <div class="metric simple">
          <div class="label">{$t.stats.simple_skipped}</div>
          <div class="value">{stats.allow_simple_count}</div>
          <div class="sub">{pct(stats.simple_ratio)}</div>
        </div>

        <div class="metric tokens">
          <div class="label">{$t.stats.tokens_in_out}</div>
          <div class="value">
            {stats.approx_tokens_from_chars_in.toLocaleString()}
            <span class="sep">/</span>
            {stats.approx_tokens_from_chars_out.toLocaleString()}
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
  {:else}
    <div class="empty">{$t.stats.loading}</div>
  {/if}
</section>

<style>
  .stats-header {
    background: var(--bg-soft);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px 22px 16px;
    margin-bottom: 20px;
  }

  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
  }

  .brand {
    font-family: var(--mono);
    font-weight: 600;
    font-size: 15px;
    color: var(--ink);
    letter-spacing: -0.01em;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .brand .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }

  .meta {
    display: flex;
    gap: 14px;
    color: var(--ink-dim);
    font-size: 12px;
    font-family: var(--mono);
  }

  .meta strong {
    color: var(--ink);
    font-weight: 600;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 14px;
    margin-bottom: 14px;
  }

  .metric {
    border-left: 2px solid var(--border);
    padding: 2px 0 2px 12px;
  }

  .metric.block {
    border-color: var(--danger);
  }
  .metric.deep {
    border-color: var(--ok);
  }
  .metric.simple {
    border-color: var(--ink-faint);
  }
  .metric.tokens {
    border-color: var(--accent);
  }
  .metric.window {
    border-color: var(--accent-dim);
  }

  .metric .label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-faint);
    margin-bottom: 2px;
  }
  .metric .value {
    font-family: var(--mono);
    font-size: 22px;
    font-weight: 600;
    color: var(--ink);
    letter-spacing: -0.02em;
  }
  .metric .value.small {
    font-size: 14px;
  }
  .metric .value .sep {
    color: var(--ink-faint);
    margin: 0 3px;
  }
  .metric .sub {
    font-size: 11px;
    color: var(--ink-faint);
    font-family: var(--mono);
  }

  .score-line {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--ink-dim);
    font-family: var(--mono);
  }

  .score-line strong {
    color: var(--ink);
    font-weight: 600;
  }

  .tag {
    display: inline-block;
    padding: 1px 8px;
    border-radius: 4px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
    margin-right: 4px;
  }
  .tag.block-tag {
    background: rgba(248, 113, 113, 0.15);
    color: var(--danger);
  }
  .tag.deep-tag {
    background: rgba(134, 239, 172, 0.15);
    color: var(--ok);
  }

  .empty {
    color: var(--ink-faint);
    font-size: 13px;
    padding: 8px 0;
  }
</style>
