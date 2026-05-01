<script lang="ts">
  import type { TelemetryRecord } from "../types";
  import { formatListLabel, formatRelative } from "../time";
  import { fmt, t } from "../i18n";
  import BreakdownPanel from "./BreakdownPanel.svelte";
  import Icon from "./Icon.svelte";

  let { record, now }: { record: TelemetryRecord; now: Date } = $props();

  let expanded = $state(false);

  // Maps the raw `decision` string to the visual class. Label text is
  // resolved reactively in the template via `$t.decision` so the badge
  // changes language without re-rendering the whole list.
  function decisionStyle(d: string): { color: string; cls: string } {
    if (d === "block") return { color: "var(--danger)", cls: "block" };
    if (d === "allow_deep_response") return { color: "var(--ok)", cls: "deep" };
    if (d === "allow_simple_task")
      return { color: "var(--ink-faint)", cls: "simple" };
    return { color: "var(--ink-dim)", cls: "other" };
  }

  const style = $derived(decisionStyle(record.decision));
  const decisionLabel = $derived.by(() => {
    if (record.decision === "block") return $t.decision.block;
    if (record.decision === "allow_deep_response") return $t.decision.deep;
    if (record.decision === "allow_simple_task") return $t.decision.skip;
    return record.decision;
  });

  function formatScore(v: number | null): string {
    return v === null ? "—" : String(v);
  }
</script>

<article class="card {style.cls}" class:expanded>
  <button class="summary" onclick={() => (expanded = !expanded)}>
    <div class="decision">
      <span
        class="tag"
        style="background: {style.color}20; color: {style.color}; border-color: {style.color}40"
      >
        {decisionLabel}
      </span>
    </div>

    <div class="time" title={record.ts}>
      <div class="primary">{formatListLabel(record.ts, $t.time, now)}</div>
      <div class="secondary">{formatRelative(record.ts, $t.time, now)}</div>
    </div>

    <div class="scores">
      <span class="score">
        <span class="score-label">c</span>
        <span class="score-value">{formatScore(record.complexity)}</span>
      </span>
      <span class="score">
        <span class="score-label">d</span>
        <span class="score-value">{formatScore(record.depth)}</span>
      </span>
    </div>

    <div class="meta">
      <span class="session" title={$t.decision.session + " " + record.session_id}>
        {record.session_id}
      </span>
      {#if record.missing_count > 0}
        <span class="missing">{fmt($t.decision.missing_n, { n: record.missing_count })}</span>
      {/if}
    </div>

    <div class="chev" class:expanded aria-hidden="true">
      <Icon name="chevron" size={14} />
    </div>
  </button>

  {#if expanded}
    <div class="details">
      <div class="info-row">
        <span>
          {$t.decision.session} <code>{record.session_id}</code>
        </span>
        <span>
          {$t.decision.profile} <code>{record.profile}</code>
        </span>
        <span>
          {$t.decision.prompt} <code>{record.prompt_chars} {$t.decision.chars}</code>
        </span>
        <span>
          {$t.decision.response} <code>{record.response_chars} {$t.decision.chars}</code>
        </span>
        {#if record.thresholds}
          <span>
            {$t.decision.thresholds}
            <code>
              c≥{record.thresholds.complexity_min_score}
              · d≥{record.thresholds.depth_min_score}
            </code>
          </span>
        {/if}
      </div>

      <div class="breakdown-grid">
        <BreakdownPanel
          breakdown={record.complexity_breakdown ?? null}
          label={$t.decision.complexity_breakdown}
        />
        {#if record.depth !== null}
          <BreakdownPanel
            breakdown={record.depth_breakdown ?? null}
            label={$t.decision.depth_breakdown}
          />
        {/if}
      </div>

      {#if record.missing_aspects && record.missing_aspects.length > 0}
        <div class="missing-section">
          <div class="missing-head">{$t.decision.missing_aspects_head}</div>
          <ul>
            {#each record.missing_aspects as m}
              <li>{m}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</article>

<style>
  .card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-left-width: 3px;
    border-radius: var(--radius-md);
    margin-bottom: var(--space-2);
    overflow: hidden;
    transition: border-color var(--duration-base) var(--ease-standard),
                box-shadow var(--duration-base) var(--ease-standard),
                transform var(--duration-base) var(--ease-standard);
  }
  .card:hover {
    border-color: var(--color-border-hover);
    box-shadow: var(--elevation-2);
    transform: translateY(-1px);
  }
  .card.block {
    border-left-color: var(--color-danger);
  }
  .card.deep {
    border-left-color: var(--color-success);
  }
  .card.simple {
    border-left-color: var(--color-border);
  }
  .card.expanded {
    border-color: var(--color-accent);
    box-shadow: var(--elevation-2);
  }

  .summary {
    width: 100%;
    background: none;
    border: none;
    color: var(--color-ink);
    display: grid;
    grid-template-columns: 70px 1fr 120px 1fr 24px;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    text-align: left;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-standard);
  }
  .summary:hover {
    background: var(--color-accent-dim);
  }

  .tag {
    display: inline-block;
    padding: 2px 10px;
    border-radius: 4px;
    border: 1px solid;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.1em;
    font-weight: 600;
  }

  .time .primary {
    font-family: var(--mono);
    font-size: 13px;
    color: var(--ink);
  }
  .time .secondary {
    font-size: 11px;
    color: var(--ink-faint);
  }

  .scores {
    display: flex;
    gap: 14px;
  }
  .score {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    font-family: var(--mono);
  }
  .score-label {
    font-size: 10px;
    color: var(--ink-faint);
    text-transform: uppercase;
  }
  .score-value {
    font-size: 15px;
    font-weight: 600;
    color: var(--ink);
    min-width: 24px;
    text-align: right;
  }

  .meta {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    align-items: center;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--ink-faint);
  }
  .session {
    background: var(--bg-hard);
    padding: 2px 6px;
    border-radius: 3px;
  }
  .missing {
    color: var(--warn);
  }

  .chev {
    color: var(--color-ink-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform var(--duration-base) var(--ease-out-quart);
  }
  .chev.expanded {
    transform: rotate(90deg);
  }

  .details {
    padding: 0 14px 14px;
    border-top: 1px solid var(--border);
    animation: slide-down 0.18s ease;
  }

  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .info-row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 18px;
    padding: 12px 0 10px;
    font-size: 12px;
    color: var(--ink-dim);
    font-family: var(--mono);
  }
  .info-row code {
    color: var(--ink);
    background: var(--bg-hard);
    padding: 1px 5px;
    border-radius: 3px;
  }

  .breakdown-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  @media (max-width: 900px) {
    .breakdown-grid {
      grid-template-columns: 1fr;
    }
  }

  .missing-section {
    margin-top: 12px;
    padding: 10px 12px;
    background: rgba(251, 191, 36, 0.06);
    border: 1px solid var(--warn);
    border-radius: 6px;
  }
  .missing-head {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--warn);
    margin-bottom: 6px;
    font-weight: 600;
  }
  .missing-section ul {
    margin: 0;
    padding-left: 18px;
    font-size: 13px;
    color: var(--ink);
  }
  .missing-section li {
    margin-bottom: 3px;
  }

  /* Responsive: stack on narrow widths */
  @media (max-width: 780px) {
    .summary {
      grid-template-columns: 60px 1fr 90px;
      grid-template-rows: auto auto;
      gap: 8px 12px;
    }
    .scores {
      grid-column: 3;
      justify-self: end;
    }
    .meta {
      grid-column: 1 / -1;
      justify-content: flex-start;
    }
    .chev {
      display: none;
    }
  }
</style>
