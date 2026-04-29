// Time formatting helpers.
// All telemetry timestamps are UTC ISO strings. The UI ALWAYS renders
// them in the user's local timezone, never as UTC. This matters because
// a decision taken at 03:37 UTC feels like 21:37 local for UTC-6, and
// the user expects to see their local wall-clock time.

import { fmt } from "./strings";

/**
 * Structural shape of the locale-specific labels this module needs.
 * Defined locally (not imported from i18n) to keep the dependency
 * direction outward: time.ts knows nothing about translations beyond
 * what the call site hands it. `$t.time` from i18n.ts satisfies this
 * shape structurally.
 */
export type TimeDict = {
  just_now: string;
  seconds_ago: string;
  minutes_ago: string;
  hours_ago: string;
  days_ago: string;
  yesterday: string;
  today: string;
};

/**
 * Format an ISO-8601 timestamp as local date + time, 24-hour.
 * Example: "2026-04-21 22:37:02"
 */
export function formatLocal(iso: string): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;

  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
    `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  );
}

/**
 * Format as a short local time only: "22:37" or "22:37:02".
 */
export function formatLocalTime(iso: string, includeSeconds = false): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  const pad = (n: number) => String(n).padStart(2, "0");
  const base = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  return includeSeconds ? `${base}:${pad(d.getSeconds())}` : base;
}

/**
 * Relative time, locale-aware. Pass `dict` from `$t.time` so the result
 * re-renders when the language changes.
 */
export function formatRelative(
  iso: string,
  dict: TimeDict,
  now: Date = new Date()
): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;

  const diffMs = now.getTime() - d.getTime();
  const sec = Math.floor(diffMs / 1000);

  // Negative values can happen if the system clock jumped backwards
  // or the timestamp is slightly in the future (clock drift across
  // machines, NAS-mounted home directories). Treat any non-positive
  // diff as "just now" rather than showing a negative duration.
  if (sec <= 10) return dict.just_now;
  if (sec < 60) return fmt(dict.seconds_ago, { n: sec });
  const min = Math.floor(sec / 60);
  if (min < 60) return fmt(dict.minutes_ago, { n: min });
  const hr = Math.floor(min / 60);
  if (hr < 24) return fmt(dict.hours_ago, { n: hr });

  // Yesterday?
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const startOfYesterday = new Date(startOfToday.getTime() - 86_400_000);
  if (d >= startOfYesterday && d < startOfToday) {
    return `${dict.yesterday} ${formatLocalTime(iso)}`;
  }

  const days = Math.floor(hr / 24);
  if (days < 7) return fmt(dict.days_ago, { n: days });

  // Older: absolute local date
  return formatLocal(iso);
}

/**
 * Short label for the list row: "today HH:MM:SS" if same day, else
 * the full local date+time.
 */
export function formatListLabel(
  iso: string,
  dict: TimeDict,
  now: Date = new Date()
): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;

  const sameDay =
    d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate();
  if (sameDay) {
    return `${dict.today} ${formatLocalTime(iso, true)}`;
  }
  return formatLocal(iso);
}
