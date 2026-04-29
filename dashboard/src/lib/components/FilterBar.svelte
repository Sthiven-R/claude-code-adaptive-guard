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
    background: var(--bg-soft);
    border: 1px solid var(--border);
    border-radius: 8px;
    margin-bottom: 14px;
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
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ink-faint);
    margin-right: 4px;
  }

  .chip {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--ink-dim);
    padding: 3px 10px;
    border-radius: 12px;
    font-size: 11px;
    font-family: var(--mono);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .chip:hover:not(.active) {
    color: var(--ink);
    border-color: var(--accent-dim);
  }

  .chip.block.active {
    background: rgba(248, 113, 113, 0.15);
    border-color: var(--danger);
    color: var(--danger);
  }
  .chip.deep.active {
    background: rgba(134, 239, 172, 0.15);
    border-color: var(--ok);
    color: var(--ok);
  }
  .chip.simple.active {
    background: rgba(154, 154, 160, 0.15);
    border-color: var(--ink-dim);
    color: var(--ink-dim);
  }

  select,
  input[type="text"] {
    background: var(--bg-hard);
    border: 1px solid var(--border);
    color: var(--ink);
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 12px;
    font-family: var(--mono);
  }
  select {
    min-width: 120px;
  }
  input[type="text"] {
    flex: 1;
    min-width: 140px;
  }

  .clear {
    background: var(--bg-hard);
    border: 1px solid var(--border);
    color: var(--ink-dim);
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 11px;
  }
  .clear:hover {
    color: var(--ink);
    border-color: var(--accent);
  }
</style>
