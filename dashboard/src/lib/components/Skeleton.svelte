<script lang="ts">
  /*
   * Skeleton placeholder. Renders a shimmering bar where real content
   * will land. Uses an indeterminate animation (no progress signal)
   * because the underlying call has no progress channel — Tauri
   * invoke() resolves once.
   *
   * Shape control: `width` / `height` accept any CSS length.
   * `variant="text"` gives the standard rounded line; `variant="block"`
   * gives a card-shaped block; `variant="circle"` gives a circle.
   */

  type Variant = "text" | "block" | "circle";

  let { width = "100%", height, variant = "text" }: {
    width?: string;
    height?: string;
    variant?: Variant;
  } = $props();

  const resolvedHeight = $derived(
    height ?? (variant === "circle" ? width : variant === "block" ? "80px" : "1em")
  );

  const radius = $derived(
    variant === "circle"
      ? "50%"
      : variant === "block"
        ? "var(--radius-md)"
        : "var(--radius-sm)"
  );
</script>

<span
  class="skeleton"
  style="width: {width}; height: {resolvedHeight}; border-radius: {radius};"
  aria-hidden="true"
></span>

<style>
  .skeleton {
    display: inline-block;
    /* Same rationale as Logo.svelte: tokens are loaded synchronously,
     * so the inline hex fallbacks were misleading in light theme. */
    background: linear-gradient(
      90deg,
      var(--color-bg-elevated) 0%,
      var(--color-bg-overlay) 50%,
      var(--color-bg-elevated) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
    vertical-align: middle;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton {
      animation: none;
      background: var(--color-bg-elevated);
    }
  }
</style>
