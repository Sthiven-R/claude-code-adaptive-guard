<script lang="ts">
  import { onDestroy } from "svelte";

  /*
   * CountUp — animates a numeric value from its previous render to
   * the new one. Cheap (one rAF loop, releases when the change is
   * applied), and respects prefers-reduced-motion: in that case it
   * snaps to the new value instantly.
   *
   * `format` lets the caller control the displayed string (locale
   * separators, decimals, etc). Default uses toLocaleString.
   *
   * Why duration 600ms by default: shorter than the human "I noticed
   * something change" threshold + long enough that the eye registers
   * the motion as ramp-up rather than a flicker.
   */

  let { value, duration = 600, format }: {
    value: number;
    duration?: number;
    format?: (n: number) => string;
  } = $props();

  // Initialised to 0 (not `value`) so the first render snaps to the
  // incoming target via the effect below, not via a captured initial.
  // svelte-check otherwise warns that `$state(value)` only reads
  // `value` once, which is technically true.
  let displayed = $state(0);
  let started = false;
  let raf: number | null = null;
  let prefersReducedMotion = $state(false);

  if (typeof window !== "undefined" && window.matchMedia) {
    prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;
  }

  $effect(() => {
    const target = value;
    if (raf !== null) cancelAnimationFrame(raf);

    // First mount: snap to the value so we do not animate 0 → target
    // every time the dashboard opens.
    if (!started) {
      displayed = target;
      started = true;
      return;
    }

    if (prefersReducedMotion || duration <= 0) {
      displayed = target;
      return;
    }

    const start = displayed;
    const delta = target - start;
    if (delta === 0) return;

    const t0 = performance.now();
    const tick = (now: number) => {
      const elapsed = now - t0;
      const k = Math.min(1, elapsed / duration);
      // Ease-out-quart for a decelerating ramp; numbers feel like
      // they "settle" instead of slamming into place.
      const eased = 1 - Math.pow(1 - k, 4);
      displayed = start + delta * eased;
      if (k < 1) {
        raf = requestAnimationFrame(tick);
      } else {
        displayed = target;
        raf = null;
      }
    };
    raf = requestAnimationFrame(tick);
  });

  onDestroy(() => {
    if (raf !== null) cancelAnimationFrame(raf);
  });

  const rendered = $derived.by(() => {
    const rounded = Math.round(displayed);
    return format ? format(rounded) : rounded.toLocaleString();
  });
</script>

<span>{rendered}</span>
