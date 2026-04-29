// Reactive filter state for the decisions list.
// Uses Svelte 5 runes in a plain module for straightforward consumption.

import type { TelemetryRecord } from "../types";

export type DecisionKind = "block" | "allow_deep_response" | "allow_simple_task";

export type TimeRange = "all" | "today" | "7d" | "1h";

export type Filters = {
  decisions: Set<DecisionKind>;
  session: string;
  timeRange: TimeRange;
};

function defaultFilters(): Filters {
  return {
    decisions: new Set<DecisionKind>([
      "block",
      "allow_deep_response",
      "allow_simple_task",
    ]),
    session: "",
    timeRange: "all",
  };
}

/**
 * Apply active filters to an array of telemetry records.
 * Pure function; returns a new filtered array.
 */
export function applyFilters(
  records: TelemetryRecord[],
  filters: Filters,
  now: Date = new Date()
): TelemetryRecord[] {
  const sessionLower = filters.session.trim().toLowerCase();
  const cutoffMs = timeCutoffMs(filters.timeRange, now);

  return records.filter((r) => {
    if (!filters.decisions.has(r.decision as DecisionKind)) {
      return false;
    }
    if (sessionLower && !r.session_id.toLowerCase().includes(sessionLower)) {
      return false;
    }
    if (cutoffMs !== null) {
      const ts = new Date(r.ts).getTime();
      if (isNaN(ts) || ts < cutoffMs) return false;
    }
    return true;
  });
}

function timeCutoffMs(range: TimeRange, now: Date): number | null {
  switch (range) {
    case "all":
      return null;
    case "1h":
      return now.getTime() - 60 * 60 * 1000;
    case "today": {
      const startOfToday = new Date(
        now.getFullYear(),
        now.getMonth(),
        now.getDate()
      );
      return startOfToday.getTime();
    }
    case "7d":
      return now.getTime() - 7 * 24 * 60 * 60 * 1000;
  }
}

export { defaultFilters };
