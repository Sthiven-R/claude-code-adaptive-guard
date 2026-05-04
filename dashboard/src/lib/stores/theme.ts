import { writable } from "svelte/store";

/*
 * Theme store.
 *
 * Three modes:
 *   - "dark"   → force dark, ignore system
 *   - "light"  → force light, ignore system
 *   - "auto"   → follow prefers-color-scheme, react to changes
 *
 * The applied attribute (data-theme on <html>) is always concrete
 * ("dark" or "light") even when the mode is "auto" — CSS does not
 * know about "auto." When the user picks "auto" we listen to the
 * media query and re-apply on every change.
 *
 * Persistence: localStorage. Tauri's WebView exposes the standard
 * Web Storage API, so this works in both `npm run dev` and the
 * bundled app. If localStorage is somehow unavailable (privacy
 * mode, embedded scenarios), the store still works — it just
 * resets every launch.
 */

export type ThemeMode = "dark" | "light" | "auto";
export type ThemeApplied = "dark" | "light";

const STORAGE_KEY = "ag.theme";
const VALID: readonly ThemeMode[] = ["dark", "light", "auto"];

function readStored(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v && (VALID as readonly string[]).includes(v)) {
      return v as ThemeMode;
    }
  } catch {
    /* localStorage may throw in privacy mode — fall through */
  }
  return "auto";
}

function persist(mode: ThemeMode): void {
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* non-fatal */
  }
}

function systemPrefersDark(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return true;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function resolve(mode: ThemeMode): ThemeApplied {
  if (mode === "auto") return systemPrefersDark() ? "dark" : "light";
  return mode;
}

function apply(applied: ThemeApplied): void {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", applied);
}

const initialMode = readStored();

export const themeMode = writable<ThemeMode>(initialMode);
export const themeApplied = writable<ThemeApplied>(resolve(initialMode));

// `writable.subscribe` fires synchronously with the current value at
// subscription time, so the apply() inside this subscriber runs on
// module load with the resolved initial mode. No need for a separate
// eager apply() — it would just be a duplicate (idempotent) call.
themeMode.subscribe((mode) => {
  persist(mode);
  const applied = resolve(mode);
  apply(applied);
  themeApplied.set(applied);
});

// React to system theme changes only when mode === "auto." In the
// other modes the user has expressed an explicit preference and we
// honor it regardless of the OS toggle.
if (typeof window !== "undefined" && window.matchMedia) {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    let current: ThemeMode = "auto";
    themeMode.subscribe((v) => (current = v))();
    if (current === "auto") {
      const applied: ThemeApplied = mq.matches ? "dark" : "light";
      apply(applied);
      themeApplied.set(applied);
    }
  };
  // Modern browsers and Tauri's WebView expose addEventListener on
  // MediaQueryList; the older addListener fallback is unnecessary.
  mq.addEventListener("change", handler);
}
