<script lang="ts">
  import type { ScoreBreakdown } from "../types";
  import { t } from "../i18n";

  let { breakdown, label }: {
    breakdown: ScoreBreakdown | null | undefined;
    label: string;
  } = $props();

  function humanizeKey(k: string): string {
    return k.replace(/_/g, " ");
  }

  function formatValue(v: unknown): string {
    if (v === null || v === undefined) return "—";
    if (typeof v === "number") return String(v);
    if (typeof v === "string") return v;
    if (Array.isArray(v)) {
      if (v.length === 0) return "[]";
      return "[" + v.map((x) => JSON.stringify(x)).join(", ") + "]";
    }
    if (typeof v === "object") return JSON.stringify(v);
    return String(v);
  }

  const nonEmptySignals = $derived.by(() => {
    if (!breakdown?.signals) return [];
    const out: [string, unknown][] = [];
    for (const [k, v] of Object.entries(breakdown.signals)) {
      // skip obvious zeros/empties unless they reveal something
      if (v === 0) continue;
      if (Array.isArray(v) && v.length === 0) continue;
      if (v && typeof v === "object" && Object.keys(v).length === 0) continue;
      out.push([k, v]);
    }
    return out;
  });
</script>

<div class="breakdown">
  <div class="head">
    <span class="label">{label}</span>
    {#if breakdown?.total !== undefined}
      <span class="total">{breakdown.total} / 100</span>
    {/if}
  </div>

  {#if !breakdown}
    <div class="muted">{$t.breakdown.not_evaluated}</div>
  {:else}
    {#if breakdown.structural !== undefined && breakdown.structural !== null}
      <div class="sub-line">
        {$t.breakdown.structural} <strong>{breakdown.structural}</strong>
        {#if breakdown.semantic !== null && breakdown.semantic !== undefined}
          · {$t.breakdown.semantic} <strong>{breakdown.semantic}</strong>
          {#if breakdown.blend_weights}
            · {$t.breakdown.blend}
            {breakdown.blend_weights.semantic.toFixed(2)} {$t.breakdown.semantic_short} +
            {breakdown.blend_weights.structural.toFixed(2)} {$t.breakdown.structural_short}
          {/if}
        {/if}
      </div>
    {/if}

    {#if breakdown.axes && Object.keys(breakdown.axes).length > 0}
      <div class="section-label">{$t.breakdown.axes_section}</div>
      <ul class="axes">
        {#each Object.entries(breakdown.axes) as [axis, pts]}
          <li>
            <span class="axis-name">{humanizeKey(axis)}</span>
            <span class="axis-pts">{pts} {$t.breakdown.pts}</span>
          </li>
        {/each}
      </ul>
    {/if}

    {#if nonEmptySignals.length > 0}
      <div class="section-label">{$t.breakdown.signals_section}</div>
      <ul class="signals">
        {#each nonEmptySignals as [key, value]}
          {#if key === "tech_tokens" && value && typeof value === "object"}
            {#each Object.entries(value as Record<string, unknown>) as [tkey, tval]}
              {#if Array.isArray(tval) && tval.length > 0}
                <li>
                  <span class="sig-name">tech.{tkey}</span>
                  <span class="sig-value">{formatValue(tval)}</span>
                </li>
              {/if}
            {/each}
          {:else}
            <li>
              <span class="sig-name">{humanizeKey(key)}</span>
              <span class="sig-value">{formatValue(value)}</span>
            </li>
          {/if}
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .breakdown {
    background: var(--bg-hard);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 6px;
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-dim);
    font-weight: 600;
  }
  .total {
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }

  .sub-line {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--ink-dim);
    margin-bottom: 8px;
  }
  .sub-line strong {
    color: var(--ink);
  }

  .muted {
    color: var(--ink-faint);
    font-size: 12px;
    padding: 4px 0;
  }

  .section-label {
    margin: 10px 0 4px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-faint);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  li {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-family: var(--mono);
    font-size: 12px;
    padding: 2px 0;
    border-bottom: 1px dotted var(--border);
    gap: 12px;
  }
  li:last-child {
    border-bottom: none;
  }

  .axis-name,
  .sig-name {
    color: var(--ink-dim);
  }
  .axis-pts {
    color: var(--ink);
    font-weight: 500;
  }
  .sig-value {
    color: var(--ink);
    word-break: break-all;
    text-align: right;
  }
</style>
