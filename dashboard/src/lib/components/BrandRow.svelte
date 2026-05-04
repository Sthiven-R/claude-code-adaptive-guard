<script lang="ts">
  import Logo from "./Logo.svelte";
  import { t } from "../i18n";

  /*
   * Top row of the dashboard: brand mark on the left, profile + version
   * meta on the right. Extracted from StatsHeader so the header becomes
   * a layout shell instead of a god component (architecture-review M-2).
   * Reused conceptually by Welcome's header — if we ever want full reuse
   * we can move that into here too, but Welcome currently needs a "first
   * run" badge instead of profile, so keep them separate for now.
   */

  let { profile, version }: {
    profile: string;
    version: string;
  } = $props();
</script>

<div class="brand-row">
  <div class="brand">
    <Logo variant="full" size={22} />
  </div>
  <div class="meta">
    <span class="profile">{$t.stats.profile} <strong>{profile}</strong></span>
    <span class="version">{version ? `v${version}` : ""}</span>
  </div>
</div>

<style>
  .brand-row {
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
</style>
