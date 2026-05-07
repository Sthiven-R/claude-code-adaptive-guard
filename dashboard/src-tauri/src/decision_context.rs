//! On-demand lookup of the prompt/response text that produced a
//! recorded decision.
//!
//! The privacy invariant of this project says we never duplicate prompt
//! or response text in our own telemetry — Claude Code already stores
//! the conversation in `~/.claude/projects/<project>/<session>.jsonl`,
//! and that is the only on-disk copy. When the operator clicks
//! "Show prompt and response" on a DecisionCard, the frontend invokes
//! this module's `get` function. We read the transcript file pointed
//! to by the telemetry record, locate the lines whose `uuid` matches
//! the recorded `prompt_uuid` / `response_uuid`, and extract the visible
//! text. Nothing is cached, nothing is written to disk.
//!
//! Failure modes return a populated `error` field instead of bubbling
//! through Tauri — the UI degrades gracefully ("transcript no longer
//! available") rather than throwing in the user's face.

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::telemetry::find_record;

/// Hard cap on how much of the transcript we are willing to read in
/// one lookup. Mirrors the 20 MB cap from `hooks/lib/transcript.py`.
/// Without this, a corrupt or maliciously-large transcript could pin
/// the dashboard process for seconds and consume gigabytes.
const MAX_TRANSCRIPT_BYTES: u64 = 20 * 1024 * 1024;

/// Hard cap per individual line. The transcript-wide cap above bounds
/// total bytes; this one bounds memory growth on any single line.
const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize)]
pub struct DecisionContext {
    pub prompt: Option<String>,
    pub response: Option<String>,
    /// Populated when we cannot resolve the context. UI shows it as
    /// "transcript no longer available" or similar — the actual string
    /// is for diagnosis, not for end-user display.
    pub error: Option<String>,
}

impl DecisionContext {
    fn err(reason: impl Into<String>) -> Self {
        DecisionContext {
            prompt: None,
            response: None,
            error: Some(reason.into()),
        }
    }
}

/// Resolve `(session_id, ts)` to the prompt/response text that produced
/// that decision, by reading the Claude Code transcript pointed to by
/// the telemetry record.
pub fn get(session_id: &str, ts: &str) -> DecisionContext {
    let record = match find_record(session_id, ts) {
        Some(r) => r,
        None => return DecisionContext::err("decision record not found"),
    };

    let path = match record.transcript_path.as_deref() {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            return DecisionContext::err(
                "this decision predates the transcript-pointer feature \
                 (Sprint 10) — no context is available",
            );
        }
    };

    if !is_safe_regular_file(&path) {
        return DecisionContext::err(
            "transcript file is missing, was rotated by Claude Code, \
             or is not a regular file",
        );
    }

    let prompt = record
        .prompt_uuid
        .as_deref()
        .and_then(|uuid| extract_visible_text(&path, "user", uuid).ok().flatten());
    let response = record
        .response_uuid
        .as_deref()
        .and_then(|uuid| extract_visible_text(&path, "assistant", uuid).ok().flatten());

    if prompt.is_none() && response.is_none() {
        return DecisionContext::err(
            "both prompt and response uuids are missing or unresolved \
             in the transcript",
        );
    }

    DecisionContext {
        prompt,
        response,
        error: None,
    }
}

/// `true` when the path exists, is a regular file, and is not a
/// symlink. Refuses device files, sockets, etc. — same defensive
/// stance as the Python side.
fn is_safe_regular_file(path: &Path) -> bool {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    meta.is_file() && !meta.file_type().is_symlink()
}

/// Open the transcript JSONL and return the visible text of the line
/// whose `type == role` and `uuid == target_uuid`. Returns Ok(None)
/// when no such line exists.
///
/// We read lines with a `BufReader` line iterator that caps each line
/// at `MAX_LINE_BYTES` and stops after `MAX_TRANSCRIPT_BYTES` of total
/// bytes have been consumed. Lines that exceed either bound are
/// silently skipped (same behavior as the Python side).
fn extract_visible_text(
    path: &Path,
    role: &str,
    target_uuid: &str,
) -> std::io::Result<Option<String>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut bytes_read: u64 = 0;
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        let line_bytes = line.len() as u64;
        bytes_read = bytes_read.saturating_add(line_bytes + 1);
        if bytes_read > MAX_TRANSCRIPT_BYTES {
            break;
        }
        if line.len() > MAX_LINE_BYTES {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let event: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if event.get("type").and_then(|v| v.as_str()) != Some(role) {
            continue;
        }
        if event.get("uuid").and_then(|v| v.as_str()) != Some(target_uuid) {
            continue;
        }

        let content = event
            .get("message")
            .and_then(|m| m.get("content"))
            .cloned()
            .unwrap_or(Value::Null);
        let text = visible_text(&content);
        if text.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(text));
    }
    Ok(None)
}

/// Mirrors `extract_text_from_blocks` in hooks/lib/transcript.py.
/// Accepts a string, a list of blocks (only `type: "text"` blocks
/// contribute), or anything else (returns "").
fn visible_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => {
            let mut parts: Vec<&str> = Vec::new();
            for block in blocks {
                let is_text = block
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == "text")
                    .unwrap_or(false);
                if !is_text {
                    continue;
                }
                if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                    parts.push(t);
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_lines(lines: &[&str]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        for l in lines {
            writeln!(f, "{}", l).unwrap();
        }
        f
    }

    #[test]
    fn extract_visible_text_returns_string_content() {
        let f = write_lines(&[
            r#"{"type":"user","uuid":"u-1","message":{"content":"hello"}}"#,
            r#"{"type":"assistant","uuid":"a-1","message":{"content":"hi"}}"#,
        ]);
        let got = extract_visible_text(f.path(), "user", "u-1").unwrap();
        assert_eq!(got.as_deref(), Some("hello"));
    }

    #[test]
    fn extract_visible_text_returns_block_array() {
        let f = write_lines(&[
            r#"{"type":"assistant","uuid":"a-2","message":{"content":[{"type":"text","text":"part one"},{"type":"thinking","text":"hidden"},{"type":"text","text":"part two"}]}}"#,
        ]);
        let got = extract_visible_text(f.path(), "assistant", "a-2").unwrap();
        assert_eq!(got.as_deref(), Some("part one\npart two"));
    }

    #[test]
    fn extract_visible_text_none_for_unknown_uuid() {
        let f = write_lines(&[
            r#"{"type":"user","uuid":"u-1","message":{"content":"hello"}}"#,
        ]);
        let got = extract_visible_text(f.path(), "user", "missing").unwrap();
        assert_eq!(got, None);
    }

    #[test]
    fn extract_visible_text_skips_role_mismatch() {
        let f = write_lines(&[
            r#"{"type":"assistant","uuid":"shared","message":{"content":"resp"}}"#,
        ]);
        let got = extract_visible_text(f.path(), "user", "shared").unwrap();
        assert_eq!(got, None);
    }

    #[test]
    fn extract_visible_text_skips_malformed_lines() {
        let f = write_lines(&[
            "not-json",
            r#"{"type":"user","uuid":"u-1","message":{"content":"hello"}}"#,
            r#"{"type":"user","uuid":"u-2"#,
        ]);
        let got = extract_visible_text(f.path(), "user", "u-1").unwrap();
        assert_eq!(got.as_deref(), Some("hello"));
    }

    #[test]
    fn visible_text_handles_image_blocks() {
        let v: Value = serde_json::from_str(
            r#"[{"type":"image","source":{"data":"abc"}},{"type":"text","text":"caption"}]"#,
        )
        .unwrap();
        assert_eq!(visible_text(&v), "caption");
    }

    #[test]
    fn visible_text_empty_for_pure_tool_use() {
        let v: Value = serde_json::from_str(r#"[{"type":"tool_use","name":"foo"}]"#).unwrap();
        assert_eq!(visible_text(&v), "");
    }

    #[test]
    fn is_safe_regular_file_rejects_missing() {
        assert!(!is_safe_regular_file(Path::new("/definitely/not/a/file/anywhere")));
    }
}
