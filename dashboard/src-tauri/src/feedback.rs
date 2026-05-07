//! User feedback storage for decisions.
//!
//! Each time the operator marks a DecisionCard as `useful` or
//! `annoying` (with an optional free-text note), we append a record to
//! `~/.claude/telemetry/adaptive-guard-feedback.jsonl`. The file is
//! deliberately separate from `adaptive-guard.jsonl`:
//!   - `adaptive-guard.jsonl` is machine-output (the guard's decisions).
//!   - `adaptive-guard-feedback.jsonl` is human-output (operator's
//!     judgments on those decisions).
//!
//! Mixing the two would couple privacy guarantees and writer
//! responsibilities that are conceptually separate. Feedback is the
//! operator's own data; they may delete this file at any time without
//! affecting the decision history.
//!
//! Append-only with latest-wins on lookup. Mutating an existing
//! feedback (toggling, editing the note) writes a NEW line whose
//! timestamp tie-breaks against earlier lines for the same
//! (session_id, decision_ts) tuple. Removing feedback writes a
//! tombstone line with `label: "cleared"` so the lookup returns "no
//! feedback" rather than the previous label.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Label values the operator can write. `Cleared` is a tombstone — it
/// means "I changed my mind, remove my prior feedback for this
/// decision". The lookup folds it into "no current feedback".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackLabel {
    Useful,
    Annoying,
    Cleared,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeedbackEntry {
    /// When the operator wrote this feedback (UTC, RFC 3339, seconds).
    pub ts: String,
    pub session_id: String,
    /// The `ts` of the telemetry record being judged.
    pub decision_ts: String,
    pub label: FeedbackLabel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// What the dashboard cares about when rendering a card: latest label
/// (or none) plus the latest note. Hides tombstones from the UI layer.
#[derive(Serialize, Default)]
pub struct FeedbackStatus {
    pub label: Option<String>,
    pub note: Option<String>,
}

/// Feedback line cap. The note field is free-text from the operator,
/// but a runaway paste would still be problematic. 4 KB is generous
/// for a one-paragraph annotation; longer notes belong in a separate
/// document, not in JSONL telemetry.
const MAX_NOTE_BYTES: usize = 4 * 1024;

fn feedback_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".claude")
            .join("telemetry")
            .join("adaptive-guard-feedback.jsonl")
    })
}

fn ensure_parent(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn restrict_perms(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_perms(_path: &std::path::Path) -> std::io::Result<()> {
    // Windows ACLs are inherited from the user's profile directory,
    // which already restricts access to the current user. The Tauri
    // bundle runs in user context, so no extra hardening is needed
    // here. Linux/macOS need the explicit chmod to avoid the file
    // ending up readable by other local users.
    Ok(())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // RFC 3339 / ISO 8601 with second precision in UTC, matching what
    // hooks/lib/telemetry.py emits. We do this without chrono to avoid
    // pulling in another dependency for one timestamp.
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    format_iso_utc(secs)
}

/// Format a unix epoch second as `YYYY-MM-DDTHH:MM:SS+00:00`.
/// Civil-calendar arithmetic mirrors RFC 3339 day-zero. Cheap enough
/// to inline rather than depend on a date crate.
fn format_iso_utc(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs_of_day = epoch_secs.rem_euclid(86_400);
    let hh = secs_of_day / 3600;
    let mm = (secs_of_day % 3600) / 60;
    let ss = secs_of_day % 60;
    let (y, m, d) = epoch_days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
        y, m, d, hh, mm, ss
    )
}

/// Convert days-since-epoch to (year, month, day). Algorithm from
/// Howard Hinnant's "date" library, unrolled for clarity.
fn epoch_days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = y + (if m <= 2 { 1 } else { 0 });
    (y, m, d)
}

/// Append one feedback record to disk. Truncates over-long notes
/// silently rather than rejecting the whole call (the note is
/// secondary; losing tail bytes is preferable to losing the label).
pub fn record(
    session_id: String,
    decision_ts: String,
    label: FeedbackLabel,
    note: Option<String>,
) -> Result<FeedbackEntry, String> {
    let path = feedback_path().ok_or_else(|| "could not resolve home directory".to_string())?;
    ensure_parent(&path).map_err(|e| format!("could not create telemetry dir: {}", e))?;

    let trimmed_note = note.map(|n| {
        if n.len() > MAX_NOTE_BYTES {
            // Truncate at a UTF-8 boundary to avoid corrupting
            // multi-byte sequences (relevant for ES notes).
            let mut end = MAX_NOTE_BYTES;
            while end > 0 && !n.is_char_boundary(end) {
                end -= 1;
            }
            n[..end].to_string()
        } else {
            n
        }
    });

    let entry = FeedbackEntry {
        ts: now_iso(),
        session_id,
        decision_ts,
        label,
        note: trimmed_note.filter(|s| !s.is_empty()),
    };

    let line = serde_json::to_string(&entry)
        .map_err(|e| format!("could not serialize entry: {}", e))?;

    let was_existing = path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("could not open feedback file: {}", e))?;
    writeln!(file, "{}", line).map_err(|e| format!("could not append: {}", e))?;
    drop(file);

    if !was_existing {
        // First-ever write. Tighten permissions on Unix-like systems.
        let _ = restrict_perms(&path);
    }

    Ok(entry)
}

/// Latest-wins lookup of the operator's current feedback for a
/// (session_id, decision_ts) tuple.
pub fn status(session_id: &str, decision_ts: &str) -> FeedbackStatus {
    let path = match feedback_path() {
        Some(p) => p,
        None => return FeedbackStatus::default(),
    };
    if !path.exists() {
        return FeedbackStatus::default();
    }
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return FeedbackStatus::default(),
    };
    let reader = BufReader::new(file);

    let mut latest: Option<FeedbackEntry> = None;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let entry: FeedbackEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.session_id != session_id || entry.decision_ts != decision_ts {
            continue;
        }
        match &latest {
            None => latest = Some(entry),
            Some(prev) if entry.ts >= prev.ts => latest = Some(entry),
            _ => {}
        }
    }

    match latest {
        None => FeedbackStatus::default(),
        Some(e) if matches!(e.label, FeedbackLabel::Cleared) => FeedbackStatus::default(),
        Some(e) => FeedbackStatus {
            label: Some(label_str(&e.label).to_string()),
            note: e.note,
        },
    }
}

fn label_str(l: &FeedbackLabel) -> &'static str {
    match l {
        FeedbackLabel::Useful => "useful",
        FeedbackLabel::Annoying => "annoying",
        FeedbackLabel::Cleared => "cleared",
    }
}

/// Write a tombstone for the (session_id, decision_ts) so subsequent
/// lookups return "no feedback". Implemented as an append rather than
/// rewriting the file, to keep the file structurally an append-only
/// log.
pub fn remove(session_id: String, decision_ts: String) -> Result<(), String> {
    record(session_id, decision_ts, FeedbackLabel::Cleared, None).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Cargo runs tests in parallel by default. `with_temp_home` mutates
    /// the process-wide HOME / USERPROFILE env vars; without serializing
    /// these tests against each other, two of them can race and read
    /// the wrong test's feedback file. The mutex bounds that race —
    /// only one HOME-mutating test runs at a time, others wait.
    static TEST_HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home<F: FnOnce()>(f: F) {
        // .lock() can return Err if a previous test panicked while
        // holding the guard. We honor the recovery: the lock is still
        // valid, we just take the inner ().
        let _guard = TEST_HOME_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::var("HOME").ok();
        let prev_userprofile = std::env::var("USERPROFILE").ok();
        std::env::set_var("HOME", dir.path());
        std::env::set_var("USERPROFILE", dir.path());
        f();
        match prev {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match prev_userprofile {
            Some(v) => std::env::set_var("USERPROFILE", v),
            None => std::env::remove_var("USERPROFILE"),
        }
    }

    #[test]
    fn record_then_status_returns_label() {
        with_temp_home(|| {
            record(
                "abc".to_string(),
                "2026-05-07T12:00:00+00:00".to_string(),
                FeedbackLabel::Useful,
                Some("good catch".to_string()),
            )
            .unwrap();
            let s = status("abc", "2026-05-07T12:00:00+00:00");
            assert_eq!(s.label.as_deref(), Some("useful"));
            assert_eq!(s.note.as_deref(), Some("good catch"));
        });
    }

    #[test]
    fn status_empty_when_no_match() {
        with_temp_home(|| {
            let s = status("never-recorded", "2026-01-01T00:00:00+00:00");
            assert!(s.label.is_none());
            assert!(s.note.is_none());
        });
    }

    #[test]
    fn record_overrides_previous_with_latest_ts() {
        with_temp_home(|| {
            // Two records for the same (session_id, decision_ts).
            // Even though the file is append-only, the second one wins.
            record(
                "s".to_string(),
                "2026-05-07T10:00:00+00:00".to_string(),
                FeedbackLabel::Useful,
                None,
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
            record(
                "s".to_string(),
                "2026-05-07T10:00:00+00:00".to_string(),
                FeedbackLabel::Annoying,
                Some("changed my mind".to_string()),
            )
            .unwrap();
            let s = status("s", "2026-05-07T10:00:00+00:00");
            assert_eq!(s.label.as_deref(), Some("annoying"));
            assert_eq!(s.note.as_deref(), Some("changed my mind"));
        });
    }

    #[test]
    fn remove_creates_tombstone_and_status_returns_none() {
        with_temp_home(|| {
            record(
                "s".to_string(),
                "2026-05-07T10:00:00+00:00".to_string(),
                FeedbackLabel::Useful,
                None,
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
            remove(
                "s".to_string(),
                "2026-05-07T10:00:00+00:00".to_string(),
            )
            .unwrap();
            let s = status("s", "2026-05-07T10:00:00+00:00");
            assert!(s.label.is_none());
            assert!(s.note.is_none());
        });
    }

    #[test]
    fn record_truncates_oversize_notes() {
        with_temp_home(|| {
            let big = "a".repeat(MAX_NOTE_BYTES + 100);
            let entry = record(
                "s".to_string(),
                "2026-05-07T10:00:00+00:00".to_string(),
                FeedbackLabel::Useful,
                Some(big),
            )
            .unwrap();
            assert_eq!(entry.note.as_deref().map(|s| s.len()), Some(MAX_NOTE_BYTES));
        });
    }

    #[test]
    fn format_iso_utc_known_epoch() {
        // Sanity: 2026-05-07T12:00:00+00:00 = 1778155200
        assert_eq!(format_iso_utc(1_778_155_200), "2026-05-07T12:00:00+00:00");
        // Epoch zero
        assert_eq!(format_iso_utc(0), "1970-01-01T00:00:00+00:00");
    }
}
