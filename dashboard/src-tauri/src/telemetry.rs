//! Read-only access to the adaptive-guard telemetry JSONL.
//!
//! The file lives at `~/.claude/telemetry/adaptive-guard.jsonl`. Each
//! line is a JSON record with the decision metadata written by
//! `hooks/lib/telemetry.py`.
//!
//! Fail-soft: on any error (path not found, permission, malformed line),
//! we return an empty result + an `error` field. Never panic. Never
//! block the UI.
//!
//! Performance: parsed records are cached behind a mutex. The cache is
//! keyed by (file_size, mtime) and invalidated automatically when the
//! file changes. Without this, every Tauri refresh would re-parse the
//! full file four times (one per command); with it, a refresh is O(1)
//! plus a tiny stat() syscall.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::SystemTime;

/// Path to `~/.claude/telemetry/adaptive-guard.jsonl`.
fn telemetry_path() -> Option<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".claude").join("telemetry").join("adaptive-guard.jsonl"))
}

fn telemetry_path_display() -> String {
    match telemetry_path() {
        Some(p) => p.display().to_string(),
        None => "(home directory not resolvable)".to_string(),
    }
}

/// Status of the telemetry file.
#[derive(Serialize)]
pub struct TelemetryStatus {
    pub path: String,
    pub exists: bool,
    pub record_count: usize,
    pub error: Option<String>,
}

/// One decision record, matching the shape written by telemetry.py.
///
/// The optional `transcript_path` / `prompt_uuid` / `response_uuid`
/// fields (added in Sprint 10) point back to the Claude Code transcript
/// that produced the decision, so the dashboard can fetch the
/// underlying text on demand. Older records (pre-Sprint-10) lack these
/// pointers — we treat that as "context not available" without erroring.
#[derive(Serialize, Deserialize, Clone)]
pub struct TelemetryRecord {
    pub ts: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub profile: String,
    #[serde(default)]
    pub decision: String,
    #[serde(default)]
    pub complexity: Option<u32>,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub missing_count: u32,
    #[serde(default)]
    pub prompt_chars: u64,
    #[serde(default)]
    pub response_chars: u64,
    #[serde(default)]
    pub thresholds: Option<serde_json::Value>,
    #[serde(default)]
    pub complexity_breakdown: Option<serde_json::Value>,
    #[serde(default)]
    pub depth_breakdown: Option<serde_json::Value>,
    #[serde(default)]
    pub missing_aspects: Option<Vec<String>>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub prompt_uuid: Option<String>,
    #[serde(default)]
    pub response_uuid: Option<String>,
}

/// Find the (cached) record matching a given (session_id, ts) tuple.
/// Returns None if no record matches. Public so other modules
/// (decision_context.rs, feedback.rs) can resolve a decision without
/// re-implementing the same lookup.
pub fn find_record(session_id: &str, ts: &str) -> Option<TelemetryRecord> {
    with_records(|records| {
        records
            .iter()
            .find(|r| r.session_id == session_id && r.ts == ts)
            .cloned()
    })
}

/// Aggregated statistics across the full telemetry set.
#[derive(Serialize, Default)]
pub struct TelemetryStats {
    pub total: usize,
    pub block_count: usize,
    pub allow_deep_count: usize,
    pub allow_simple_count: usize,
    pub block_ratio: f64,
    pub deep_ratio: f64,
    pub simple_ratio: f64,
    pub avg_complexity_block: f64,
    pub avg_depth_block: f64,
    pub avg_missing_block: f64,
    pub avg_complexity_deep: f64,
    pub avg_depth_deep: f64,
    /// Approximate token counters. This is literally `chars / 4` — a
    /// rough heuristic that diverges for non-Latin scripts (Chinese,
    /// Japanese, Thai: closer to 1 char/token; Spanish/Portuguese:
    /// closer to 3 chars/token). Named explicitly so callers know this
    /// is not a real tokenizer count.
    pub approx_tokens_from_chars_in: u64,
    pub approx_tokens_from_chars_out: u64,
    pub first_ts: Option<String>,
    pub last_ts: Option<String>,
}

/// Histogram bucket for a score dimension.
#[derive(Serialize)]
pub struct HistogramBucket {
    pub bucket_lo: u32,
    pub bucket_hi: u32,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

/// What we cache. `key` is (file_size, mtime) from the last successful
/// load. The records vector is wrapped in an `Arc` so each call to
/// `with_records` can hand the closure a cheap pointer-clone instead
/// of a full deep clone of every record. With ~10k records (the
/// rotation cap) the deep clone was ~5–15 ms per call, multiplied by
/// six commands per refresh; the Arc clone is constant-time.
struct Cache {
    key: Option<(u64, SystemTime)>,
    records: Arc<Vec<TelemetryRecord>>,
}

// `Arc::new()` is not const-callable in stable Rust, so we wrap the
// static in `LazyLock` (stable since 1.80). The first cache access
// triggers initialization; subsequent accesses are an atomic load.
static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| {
    Mutex::new(Cache {
        key: None,
        records: Arc::new(Vec::new()),
    })
});

/// Compute the current identity key for the telemetry file. If the file
/// does not exist or cannot be stat'd, returns None.
fn current_key() -> Option<(u64, SystemTime)> {
    let p = telemetry_path()?;
    let m = fs::metadata(&p).ok()?;
    Some((m.len(), m.modified().ok()?))
}

/// Apply `f` to the latest parsed records, reloading from disk only if
/// the file changed since the last call. `f` receives a snapshot Arc
/// so the mutex is released before `f` runs and the caller's view of
/// the data cannot be mutated mid-iteration even if a concurrent
/// command swaps the cached Arc.
fn with_records<T, F: FnOnce(&[TelemetryRecord]) -> T>(f: F) -> T {
    let current = current_key();

    let snapshot: Arc<Vec<TelemetryRecord>> = {
        let mut guard = CACHE.lock().unwrap();
        let stale = match (guard.key, current) {
            (Some(old), Some(new)) => old != new,
            _ => true,
        };
        if stale {
            guard.records = Arc::new(load_from_disk());
            guard.key = current;
        }
        Arc::clone(&guard.records)
    };

    f(&snapshot)
}

fn load_from_disk() -> Vec<TelemetryRecord> {
    let path = match telemetry_path() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut records: Vec<TelemetryRecord> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(rec) = serde_json::from_str::<TelemetryRecord>(trimmed) {
            records.push(rec);
        }
    }
    records
}

/// External hook for the watcher to explicitly invalidate the cache when
/// it knows the file has changed. A redundant belt-and-suspenders with
/// the mtime-based invalidation — the stat check will also catch it,
/// but invalidating eagerly means the next Tauri command does a full
/// reparse instead of two stat syscalls + reparse.
pub fn invalidate_cache() {
    let mut guard = CACHE.lock().unwrap();
    guard.key = None;
    guard.records = Arc::new(Vec::new());
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Status of the telemetry file. Uses the cache — no extra read.
pub fn status() -> TelemetryStatus {
    let path_display = telemetry_path_display();
    let path = match telemetry_path() {
        Some(p) => p,
        None => {
            return TelemetryStatus {
                path: path_display,
                exists: false,
                record_count: 0,
                error: Some("could not resolve home directory".to_string()),
            };
        }
    };
    if !path.exists() {
        return TelemetryStatus {
            path: path_display,
            exists: false,
            record_count: 0,
            error: None,
        };
    }
    let count = with_records(|r| r.len());
    TelemetryStatus {
        path: path_display,
        exists: true,
        record_count: count,
        error: None,
    }
}

pub fn recent(limit: usize) -> Vec<TelemetryRecord> {
    with_records(|records| {
        if records.len() > limit {
            records[records.len() - limit..].to_vec()
        } else {
            records.to_vec()
        }
    })
}

pub fn stats() -> TelemetryStats {
    with_records(|records| {
        let total = records.len();
        if total == 0 {
            return TelemetryStats::default();
        }

        let mut block: Vec<&TelemetryRecord> = Vec::new();
        let mut deep: Vec<&TelemetryRecord> = Vec::new();
        let mut simple: usize = 0;
        let mut total_prompt_chars: u64 = 0;
        let mut total_response_chars: u64 = 0;

        for r in records {
            total_prompt_chars += r.prompt_chars;
            total_response_chars += r.response_chars;
            match r.decision.as_str() {
                "block" => block.push(r),
                "allow_deep_response" => deep.push(r),
                "allow_simple_task" => simple += 1,
                _ => {}
            }
        }

        let bc = block.len();
        let dc = deep.len();

        fn avg_opt(
            records: &[&TelemetryRecord],
            f: impl Fn(&TelemetryRecord) -> Option<u32>,
        ) -> f64 {
            // Single-pass fold: avoids materializing the intermediate
            // Vec<u32> the previous version allocated. With ~10k
            // records this saves a small allocation per call, but more
            // importantly removes a paint of garbage the allocator
            // would have to collect.
            let (sum, count) = records.iter().filter_map(|r| f(r)).fold(
                (0u64, 0usize),
                |(s, c), v| (s + v as u64, c + 1),
            );
            if count == 0 {
                0.0
            } else {
                sum as f64 / count as f64
            }
        }
        fn avg_u32(records: &[&TelemetryRecord], f: impl Fn(&TelemetryRecord) -> u32) -> f64 {
            if records.is_empty() {
                0.0
            } else {
                records.iter().map(|r| f(r) as f64).sum::<f64>() / records.len() as f64
            }
        }

        let first_ts = records.first().map(|r| r.ts.clone());
        let last_ts = records.last().map(|r| r.ts.clone());

        TelemetryStats {
            total,
            block_count: bc,
            allow_deep_count: dc,
            allow_simple_count: simple,
            block_ratio: bc as f64 / total as f64,
            deep_ratio: dc as f64 / total as f64,
            simple_ratio: simple as f64 / total as f64,
            avg_complexity_block: avg_opt(&block, |r| r.complexity),
            avg_depth_block: avg_opt(&block, |r| r.depth),
            avg_missing_block: avg_u32(&block, |r| r.missing_count),
            avg_complexity_deep: avg_opt(&deep, |r| r.complexity),
            avg_depth_deep: avg_opt(&deep, |r| r.depth),
            approx_tokens_from_chars_in: total_prompt_chars / 4,
            approx_tokens_from_chars_out: total_response_chars / 4,
            first_ts,
            last_ts,
        }
    })
}

/// Histogram in 10-point buckets.
///
/// IMPORTANT: scores can be exactly 100 (structural/complexity/depth
/// total is clamped to `min(100, ...)`). A naive `(score / 10) * 10`
/// puts 100 in its own 100-109 bucket that the frontend does not render
/// — silently losing data. Fold 100 into the 90-bucket.
pub fn histogram(dim: &str) -> Vec<HistogramBucket> {
    with_records(|records| {
        let mut buckets: BTreeMap<u32, usize> = BTreeMap::new();
        for r in records {
            let v = match dim {
                "complexity" => r.complexity,
                "depth" => r.depth,
                _ => None,
            };
            if let Some(score) = v {
                let clamped = score.min(99);
                let b = (clamped / 10) * 10;
                *buckets.entry(b).or_insert(0) += 1;
            }
        }
        buckets
            .into_iter()
            .map(|(lo, count)| HistogramBucket {
                bucket_lo: lo,
                bucket_hi: lo + 9,
                count,
            })
            .collect()
    })
}
