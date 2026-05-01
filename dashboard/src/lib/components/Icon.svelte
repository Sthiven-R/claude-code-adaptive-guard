<script lang="ts">
  /*
   * Adaptive Guard icon set.
   *
   * Six icons drawn in-house, kept stylistically consistent:
   *   - 24x24 viewBox
   *   - stroke 1.75 (logo is 2.25–2.5; UI icons are slightly lighter
   *     so they read as chrome, not as brand marks)
   *   - round caps + joins
   *   - currentColor stroke (callers tint with CSS color)
   *   - no fills (kept hairline / line-art style)
   *
   * Why a single component instead of 6 files: each icon is small
   * (one or two paths), and centralizing them makes it trivial to
   * audit consistency. If we ever ship more than ~12 icons, switch
   * to per-file + a barrel export.
   *
   * Adding a new icon: extend the `IconName` union, add a case to
   * the template, keep the same stroke-width / cap / join. If your
   * icon needs a different weight, it does not belong here.
   */

  export type IconName =
    | "gear"
    | "chevron"
    | "close"
    | "refresh"
    | "install"
    | "trash";

  let { name, size = 16, label, strokeWidth = 1.75 }: {
    name: IconName;
    size?: number;
    /** Accessible label. Pass null for purely decorative icons. */
    label?: string | null;
    strokeWidth?: number;
  } = $props();

  // Decorative when no label is given. Either an aria-label is set
  // or aria-hidden is true — never both, never neither.
  const role = $derived(label ? "img" : undefined);
  const ariaHidden = $derived(label ? undefined : "true");
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  {role}
  aria-label={label ?? undefined}
  aria-hidden={ariaHidden}
>
  {#if label}<title>{label}</title>{/if}

  {#if name === "gear"}
    <!--
      Geometric gear: center circle + 8 ticks (4 cardinal + 4 diagonal).
      Reads as "settings" without the busy curves of the Material gear.
    -->
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.93 4.93l2.12 2.12M16.95 16.95l2.12 2.12M19.07 4.93l-2.12 2.12M7.05 16.95l-2.12 2.12" />
  {:else if name === "chevron"}
    <!-- Right-pointing chevron. Rotate via CSS transform when needed. -->
    <polyline points="9 6 15 12 9 18" />
  {:else if name === "close"}
    <!-- An honest × — two lines, no flair. -->
    <line x1="6" y1="6" x2="18" y2="18" />
    <line x1="18" y1="6" x2="6" y2="18" />
  {:else if name === "refresh"}
    <!--
      Three-quarter loop with an arrowhead. Keep the open quadrant on
      the top-right so the icon has visual rhythm with the logo
      (which is also open at the top-right).
    -->
    <path d="M21 12a9 9 0 1 1-3-6.7L21 8" />
    <polyline points="21 3 21 8 16 8" />
  {:else if name === "install"}
    <!-- Down-arrow onto a tray. Reads as "install" / "download". -->
    <path d="M12 3v12" />
    <polyline points="7 10 12 15 17 10" />
    <line x1="5" y1="20" x2="19" y2="20" />
  {:else if name === "trash"}
    <polyline points="3 6 5 6 21 6" />
    <path d="M19 6l-1.5 14a2 2 0 0 1-2 1.8h-7a2 2 0 0 1-2-1.8L5 6" />
    <path d="M10 11v6M14 11v6" />
  {/if}
</svg>

<style>
  svg {
    display: inline-block;
    flex-shrink: 0;
    vertical-align: middle;
  }
</style>
