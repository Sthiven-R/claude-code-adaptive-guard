<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type {
    HistogramBucket,
    HookStatus,
    TelemetryRecord,
    TelemetryStats,
    TelemetryStatus,
  } from "./lib/types";
  import { defaultFilters, applyFilters, type Filters } from "./lib/stores/filters";
  import { fmt, t } from "./lib/i18n";
  import StatsHeader from "./lib/components/StatsHeader.svelte";
  import DecisionCard from "./lib/components/DecisionCard.svelte";
  import ScoreHistogram from "./lib/components/ScoreHistogram.svelte";
  import FilterBar from "./lib/components/FilterBar.svelte";
  import LiveIndicator from "./lib/components/LiveIndicator.svelte";
  import Welcome from "./lib/components/Welcome.svelte";
  import SettingsModal from "./lib/components/SettingsModal.svelte";
  import Icon from "./lib/components/Icon.svelte";

  // Version is fetched from the Tauri backend (CARGO_PKG_VERSION) so
  // there is one source of truth for the four version-bearing files
  // (VERSION, package.json, Cargo.toml, tauri.conf.json). Empty string
  // until onMount resolves; UI labels that bind to it just render an
  // empty space for ~10ms on first paint.
  let version = $state("");

  let status: TelemetryStatus | null = $state(null);
  let stats: TelemetryStats | null = $state(null);
  let recent: TelemetryRecord[] = $state([]);
  let complexityHist: HistogramBucket[] = $state([]);
  let depthHist: HistogramBucket[] = $state([]);
  let hook: HookStatus | null = $state(null);
  let loading = $state(true);
  let errorMsg: string | null = $state(null);
  let showingLimit = $state(50);
  let live = $state(false);
  let welcomeSkipped = $state(false);
  let settingsOpen = $state(false);

  let filters: Filters = $state(defaultFilters());

  let unlisten: UnlistenFn | null = null;

  // Hoisted "now" — passed down to formatRelative / formatListLabel
  // call sites so each visible card doesn't allocate its own Date per
  // render. Ticks every 30 s so relative labels ("5m ago") stay current
  // without the user having to refresh.
  let now: Date = $state(new Date());
  let nowTimer: ReturnType<typeof setInterval> | null = null;

  const activeProfile = $derived(recent[recent.length - 1]?.profile ?? "balanced");
  const complexityThreshold = $derived<number | null>(
    recent[recent.length - 1]?.thresholds?.complexity_min_score ?? 40
  );
  const depthThreshold = $derived<number | null>(
    recent[recent.length - 1]?.thresholds?.depth_min_score ?? 40
  );

  const displayRecords = $derived(
    applyFilters([...recent].reverse(), filters)
  );

  // Welcome shows on a fresh install: no hook, no telemetry, not skipped.
  // Once the user clicks "Skip welcome" it stays hidden for the session
  // (welcomeSkipped is not persisted — it resets next launch, which is
  // fine because next launch will see hook.installed === true).
  const showWelcome = $derived.by(() => {
    if (welcomeSkipped) return false;
    if (!hook || hook.installed) return false;
    if (!status || status.exists) return false;
    return true;
  });

  async function refresh() {
    try {
      errorMsg = null;
      const [s, st, r, ch, dh, hk] = await Promise.all([
        invoke<TelemetryStatus>("telemetry_status"),
        invoke<TelemetryStats>("telemetry_stats"),
        invoke<TelemetryRecord[]>("telemetry_recent", { limit: showingLimit }),
        invoke<HistogramBucket[]>("telemetry_histogram", { dim: "complexity" }),
        invoke<HistogramBucket[]>("telemetry_histogram", { dim: "depth" }),
        invoke<HookStatus>("hook_status"),
      ]);
      status = s;
      stats = st;
      recent = r;
      complexityHist = ch;
      depthHist = dh;
      hook = hk;
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    // Fire version fetch in parallel with the first refresh; whichever
    // resolves first paints — version is purely cosmetic.
    invoke<string>("app_version")
      .then((v) => (version = v))
      .catch(() => {
        /* leave empty — footer just renders without it */
      });
    await refresh();
    try {
      // The Rust watcher already coalesces filesystem events with a
      // 250ms debounce (see watcher.rs::spawn). Adding a second JS-side
      // debounce here would only push the visible latency higher
      // without protecting against any failure mode the backend
      // doesn't already handle.
      unlisten = await listen("telemetry-changed", () => {
        refresh();
      });
      live = true;
    } catch {
      live = false;
    }
    nowTimer = setInterval(() => {
      now = new Date();
    }, 30_000);
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (nowTimer) clearInterval(nowTimer);
  });

  function loadMore() {
    showingLimit += 50;
    refresh();
  }
</script>

<main>
  <button
    class="gear"
    onclick={() => (settingsOpen = true)}
    aria-label={$t.app.settings_label}
    title={$t.app.settings_label}
  >
    <Icon name="gear" size={16} />
  </button>

  {#if showWelcome && hook}
    <Welcome
      status={hook}
      onInstalled={refresh}
      onSkip={() => (welcomeSkipped = true)}
    />
  {:else}
    <StatsHeader {stats} profile={activeProfile} {version} {now} />

    {#if errorMsg}
      <div class="error">
        <strong>{$t.app.error}</strong>
        <code>{errorMsg}</code>
      </div>
    {/if}

    {#if status && !status.exists}
      <div class="warn">
        <strong>{$t.app.no_telemetry_yet}</strong>
        <div class="sub">
          {#if hook?.installed}
            {$t.app.hook_installed_no_decisions}
          {:else}
            {$t.app.hook_not_installed_lead}
            <button class="inline-link" onclick={() => (settingsOpen = true)}>{$t.app.hook_not_installed_link}</button>
            {$t.app.hook_not_installed_or}
            <code>adaptive-guard install</code>{$t.app.hook_not_installed_then}
          {/if}
        </div>
        <div class="path-info">
          {$t.app.looking_at} <code>{status.path}</code>
        </div>
      </div>
    {/if}

    <div class="charts">
    <ScoreHistogram
      buckets={complexityHist}
      title={$t.histogram.complexity_distribution}
      threshold={complexityThreshold}
    />
    <ScoreHistogram
      buckets={depthHist}
      title={$t.histogram.depth_distribution}
      threshold={depthThreshold}
    />
  </div>

  <FilterBar bind:filters />

  <section class="recent-section">
    <div class="section-head">
      <h2>
        {$t.app.recent_decisions}
        <LiveIndicator active={live} />
      </h2>
      <div class="actions">
        <span class="count">
          {displayRecords.length} {$t.app.shown}
          {#if recent.length !== displayRecords.length}
            <span class="of">{fmt($t.app.of_loaded, { n: recent.length })}</span>
          {/if}
        </span>
        <button onclick={refresh} disabled={loading}>
          {loading ? $t.app.loading : $t.app.refresh}
        </button>
      </div>
    </div>

    {#if displayRecords.length === 0}
      <div class="empty">
        <!--
          Empty illustration: a faint waveform that mirrors the logo
          mark, signaling "the channel is open, no signal yet" rather
          than "broken". Decorative; aria-hidden because the surround
          text already conveys the state.
        -->
        <svg
          class="empty-art"
          width="140"
          height="56"
          viewBox="0 0 140 56"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <line x1="14" y1="46" x2="126" y2="46" opacity="0.18" />
          <line x1="14" y1="32" x2="126" y2="32" opacity="0.28" stroke-dasharray="4 5" />
          <path d="M 14 16 L 38 16 L 48 6 L 68 26 L 78 16 L 126 16" opacity="0.55" />
        </svg>
        <p class="empty-text">
          {#if recent.length === 0}
            {$t.app.no_decisions_yet}
          {:else}
            {$t.app.no_match_filters}
          {/if}
        </p>
      </div>
    {:else}
      <div class="list">
        <!--
          Key includes array index because ts+session_id can collide
          when two Stop events fire within the same second in the same
          session. Adding the index disambiguates cleanly without
          needing a schema change to add a sequence number.
        -->
        {#each displayRecords as r, i (r.ts + "|" + r.session_id + "|" + i)}
          <DecisionCard record={r} {now} />
        {/each}
      </div>

      {#if stats && recent.length < stats.total}
        <div class="load-more">
          <button onclick={loadMore} disabled={loading}>
            {fmt($t.app.load_more, { n: stats.total - recent.length })}
          </button>
        </div>
      {/if}
    {/if}
  </section>

    <footer>
      <span>adaptive-guard{version ? ` v${version}` : ""}</span>
      <span class="sep">·</span>
      <span>{fmt($t.app.loaded_n_of_total, { n: recent.length, total: stats?.total ?? 0 })}</span>
      <span class="sep">·</span>
      <span>{$t.app.minimize_to_tray}</span>
    </footer>
  {/if}
</main>

{#if settingsOpen && hook}
  <SettingsModal
    {hook}
    {version}
    onClose={() => (settingsOpen = false)}
    onChanged={refresh}
  />
{/if}

<style>
  main {
    padding: 20px 24px 40px;
    max-width: 1280px;
    margin: 0 auto;
    height: 100vh;
    overflow-y: auto;
    position: relative;
  }

  .gear {
    position: absolute;
    top: 24px;
    right: 28px;
    background: transparent;
    border: 1px solid transparent;
    color: var(--color-ink-faint);
    padding: 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    z-index: 10;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: color var(--duration-fast) var(--ease-standard),
                border-color var(--duration-fast) var(--ease-standard),
                background var(--duration-fast) var(--ease-standard),
                transform var(--duration-fast) var(--ease-standard);
  }
  .gear:hover {
    color: var(--color-ink);
    border-color: var(--color-border);
    background: var(--color-bg-elevated);
  }
  .gear:active {
    transform: scale(0.94);
  }

  .inline-link {
    background: none;
    border: none;
    color: var(--accent);
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
    font: inherit;
  }
  .inline-link:hover {
    opacity: 0.8;
  }

  .error {
    background: rgba(248, 113, 113, 0.1);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: 10px 14px;
    border-radius: 6px;
    margin-bottom: 16px;
  }

  .warn {
    background: rgba(251, 191, 36, 0.06);
    border: 1px solid var(--warn);
    color: var(--ink);
    padding: 14px 18px;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  .warn strong {
    color: var(--warn);
  }
  .warn .sub {
    color: var(--ink-dim);
    font-size: 13px;
    margin-top: 4px;
    line-height: 1.5;
  }
  .warn .sub code {
    background: var(--bg-hard);
    padding: 1px 6px;
    border-radius: 3px;
    color: var(--accent);
  }
  .warn .path-info {
    margin-top: 8px;
    font-size: 12px;
    color: var(--ink-faint);
  }
  .warn .path-info code {
    color: var(--ink-dim);
    background: none;
    padding: 0;
  }

  .charts {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    margin-bottom: 14px;
  }
  @media (max-width: 900px) {
    .charts {
      grid-template-columns: 1fr;
    }
  }

  .recent-section {
    margin-bottom: 20px;
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 10px;
  }
  h2 {
    margin: 0;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ink-dim);
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .count {
    font-size: 11px;
    color: var(--ink-faint);
    font-family: var(--mono);
  }
  .count .of {
    opacity: 0.7;
    margin-left: 4px;
  }

  button {
    background: var(--bg-soft);
    border: 1px solid var(--border);
    color: var(--ink);
    padding: 5px 12px;
    border-radius: 6px;
    font-size: 12px;
    transition: all 0.15s ease;
  }
  button:hover:not(:disabled) {
    background: var(--accent-dim);
    border-color: var(--accent);
    color: var(--ink);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty {
    color: var(--color-ink-faint);
    font-size: var(--text-small);
    text-align: center;
    padding: var(--space-10) var(--space-4);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }
  .empty-art {
    color: var(--color-accent);
    opacity: 0.85;
  }
  .empty-text {
    margin: 0;
    max-width: 340px;
    line-height: var(--leading-relaxed);
  }

  .load-more {
    display: flex;
    justify-content: center;
    padding: 14px 0;
  }

  footer {
    display: flex;
    justify-content: center;
    gap: 10px;
    padding: 20px 0 8px;
    color: var(--ink-faint);
    font-family: var(--mono);
    font-size: 11px;
    flex-wrap: wrap;
  }
  footer .sep {
    opacity: 0.5;
  }
</style>
