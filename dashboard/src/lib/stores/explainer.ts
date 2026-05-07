import { writable } from "svelte/store";

/*
 * Explainer panel store.
 *
 * Tracks whether the "How to read this dashboard" panel at the top of
 * the window is expanded. First-time users see it open; once they
 * collapse it, the choice persists across launches via localStorage.
 *
 * Persistence key: `ag.explainer-expanded`. Falsy reads (private mode,
 * Tauri sandbox quirks) fall back to the default of `true` — the
 * worst case is a fresh-feeling first launch every time, never a
 * silently broken UI.
 */

const STORAGE_KEY = "ag.explainer-expanded";
const DEFAULT_EXPANDED = true;

function readStored(): boolean {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === null) return DEFAULT_EXPANDED;
    return v === "true";
  } catch {
    /* localStorage unavailable — default-on is the safe choice */
    return DEFAULT_EXPANDED;
  }
}

export const explainerExpanded = writable<boolean>(readStored());

explainerExpanded.subscribe((expanded) => {
  try {
    localStorage.setItem(STORAGE_KEY, String(expanded));
  } catch {
    /* non-fatal */
  }
});
