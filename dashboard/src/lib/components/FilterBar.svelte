<script lang="ts">
  import type { DecisionKind, Filters, TimeRange } from "../stores/filters";
  import { t } from "../i18n";

  let { filters = $bindable() }: { filters: Filters } = $props();

  function toggleDecision(kind: DecisionKind) {
    const next = new Set(filters.decisions);
    if (next.has(kind)) {
      if (next.size > 1) next.delete(kind);
    } else {
      next.add(kind);
    }
    filters = { ...filters, decisions: next };
  }

  function setTimeRange(r: TimeRange) {
    filters = { ...filters, timeRange: r };
  }

  function onSessionInput(e: Event) {
    const v = (e.target as HTMLInputElement).value;
    filters = { ...filters, session: v };
  }

  function clearAll() {
    filters = {
      decisions: new Set<DecisionKind>([
        "block",
        "allow_deep_response",
        "allow_simple_task",
      ]),
      session: "",
      timeRange: "all",
    };
  }
</script>

<div class="filter-bar">
  <div class="group">
    <span class="label">{$t.filter.decision}</span>
    <button
      class="chip block"
      class:active={filters.decisions.has("block")}
      onclick={() => toggleDecision("block")}
    >
      {$t.filter.block}
    </button>
    <button
      class="chip deep"
      class:active={filters.decisions.has("allow_deep_response")}
      onclick={() => toggleDecision("allow_deep_response")}
    >
      {$t.filter.deep}
    </button>
    <button
      class="chip simple"
      class:active={filters.decisions.has("allow_simple_task")}
      onclick={() => toggleDecision("allow_simple_task")}
    >
      {$t.filter.simple}
    </button>
  </div>

  <div class="group">
    <span class="label">{$t.filter.time}</span>
    <select
      value={filters.timeRange}
      onchange={(e) =>
        setTimeRange(
          (e.currentTarget as HTMLSelectElement).value as TimeRange
        )}
    >
      <option value="all">{$t.filter.all_time}</option>
      <option value="7d">{$t.filter.last_7_days}</option>
      <option value="today">{$t.filter.today}</option>
      <option value="1h">{$t.filter.last_hour}</option>
    </select>
  </div>

  <div class="group grow">
    <span class="label">{$t.filter.session}</span>
    <input
      type="text"
      placeholder={$t.filter.session_placeholder}
      value={filters.session}
      oninput={onSessionInput}
    />
  </div>

  <button class="clear" onclick={clearAll}>{$t.filter.clear}</button>
</div>

<style>
  .filter-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    align-items: center;
    padding: 10px 14px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-3);
  }

  .group {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .group.grow {
    flex: 1;
    min-width: 180px;
  }

  .label {
    font-size: var(--text-mono-micro);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
    color: var(--color-ink-faint);
    margin-right: 4px;
  }

  .chip {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-ink-dim);
    padding: 3px 10px;
    border-radius: var(--radius-lg);
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    transition: color var(--duration-fast) var(--ease-standard),
                border-color var(--duration-fast) var(--ease-standard),
                background var(--duration-fast) var(--ease-standard);
  }
  .chip:hover:not(.active) {
    color: var(--color-ink);
    border-color: var(--color-accent-dim);
  }

  .chip.block.active {
    background: var(--color-danger-soft);
    border-color: var(--color-danger);
    color: var(--color-danger);
  }
  .chip.deep.active {
    background: var(--color-success-soft);
    border-color: var(--color-success);
    color: var(--color-success);
  }
  .chip.simple.active {
    background: var(--color-ink-soft);
    border-color: var(--color-ink-dim);
    color: var(--color-ink-dim);
  }

  select,
  input[type="text"] {
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    color: var(--color-ink);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--text-small);
    font-family: var(--font-mono);
  }
  select {
    min-width: 120px;
  }
  input[type="text"] {
    flex: 1;
    min-width: 140px;
  }

  .clear {
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    color: var(--color-ink-dim);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: var(--text-micro);
    transition: color var(--duration-fast) var(--ease-standard),
                border-color var(--duration-fast) var(--ease-standard);
  }
  .clear:hover {
    color: var(--color-ink);
    border-color: var(--color-accent);
  }
</style>
