// Shared types mirroring the Rust backend shapes.

export type TelemetryStatus = {
  path: string;
  exists: boolean;
  record_count: number;
  error: string | null;
};

export type ScoreBreakdown = {
  total?: number;
  structural?: number | null;
  semantic?: number | null;
  blend_weights?: { semantic: number; structural: number } | null;
  axes?: Record<string, number>;
  signals?: Record<string, unknown>;
};

export type TelemetryRecord = {
  ts: string;
  session_id: string;
  profile: string;
  decision: "block" | "allow_deep_response" | "allow_simple_task" | string;
  complexity: number | null;
  depth: number | null;
  missing_count: number;
  prompt_chars: number;
  response_chars: number;
  thresholds?: { complexity_min_score?: number; depth_min_score?: number } | null;
  complexity_breakdown?: ScoreBreakdown | null;
  depth_breakdown?: ScoreBreakdown | null;
  missing_aspects?: string[] | null;
};

export type TelemetryStats = {
  total: number;
  block_count: number;
  allow_deep_count: number;
  allow_simple_count: number;
  block_ratio: number;
  deep_ratio: number;
  simple_ratio: number;
  avg_complexity_block: number;
  avg_depth_block: number;
  avg_missing_block: number;
  avg_complexity_deep: number;
  avg_depth_deep: number;
  // NOTE: these are literally chars/4. Not a real tokenizer count;
  // diverges from actual tokens for non-Latin scripts. Display must
  // label them accordingly.
  approx_tokens_from_chars_in: number;
  approx_tokens_from_chars_out: number;
  first_ts: string | null;
  last_ts: string | null;
};

export type HistogramBucket = {
  bucket_lo: number;
  bucket_hi: number;
  count: number;
};

export type HookStatus = {
  installed: boolean;
  repo_root: string | null;
  config_path: string;
  settings_path: string;
  error: string | null;
};

export type InstallResult = {
  ok: boolean;
  message: string;
  backup_path: string | null;
};
