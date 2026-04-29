"""
telemetry.py - Anonymized decision logging with breakdown for explainability.

Each record includes the axis-by-axis BREAKDOWN of how complexity and
depth were scored, so users can inspect any decision they see in the
dashboard.

PRIVACY CONTRACT (honestly honored):
  - Only integers, floats, enum strings, and NAMED COUNTS are logged.
  - We do NOT log prompt text.
  - We do NOT log response text.
  - We do NOT log matched substrings (paths, URLs, inline code, tokens).
    Only their counts are kept. This is the fix for the audit finding
    that matched paths/URLs routinely contain user secrets (API tokens,
    home-dir credentials). If you see a string in here that could
    originate from user input, that is a privacy bug — file an issue.

Log location: ~/.claude/telemetry/adaptive-guard.jsonl
Rotation: keeps last 10000 decisions using a streaming tail + atomic
replace. Uses a deque so we never load the whole file into memory.

Test isolation: if env var ADAPTIVE_GUARD_TELEMETRY_DISABLED=1 is set,
no records are written. The test runner sets this by default so tests
never pollute real telemetry.
"""
from __future__ import annotations

import json
import os
import tempfile
import traceback
from collections import deque
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

MAX_LOG_LINES = 10000
ROTATE_BYTES = 5 * 1024 * 1024
LOG_DIR = Path.home() / ".claude" / "telemetry"
LOG_FILE = LOG_DIR / "adaptive-guard.jsonl"
ERR_LOG_FILE = LOG_DIR / "adaptive-guard.err.log"


def _is_disabled_by_env() -> bool:
    return os.environ.get("ADAPTIVE_GUARD_TELEMETRY_DISABLED", "") == "1"


def log_decision(
    cfg: dict[str, Any],
    hook_input: dict[str, Any],
    complexity: int | None,
    depth: int | None,
    decision: str,
    missing_count: int = 0,
    complexity_breakdown: dict | None = None,
    depth_breakdown: dict | None = None,
    missing_aspects: list[str] | None = None,
    prompt_chars: int = 0,
    response_chars: int = 0,
) -> None:
    """Write a single anonymized decision record with full breakdown."""
    if _is_disabled_by_env():
        return
    if not cfg.get("telemetry_enabled", True):
        return

    try:
        LOG_DIR.mkdir(parents=True, exist_ok=True)

        # Defensive: coerce session_id to str before slicing (audit finding:
        # non-string session_id crashes downstream).
        sid_raw = hook_input.get("session_id", "unknown")
        session_id = str(sid_raw)[:8] if sid_raw is not None else "unknown"

        record: dict[str, Any] = {
            "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "session_id": session_id,
            "profile": cfg.get("profile", "unknown"),
            "decision": decision,
            "complexity": complexity,
            "depth": depth,
            "missing_count": missing_count,
            "prompt_chars": prompt_chars,
            "response_chars": response_chars,
            "thresholds": cfg.get("thresholds", {}),
        }

        if complexity_breakdown is not None:
            record["complexity_breakdown"] = complexity_breakdown
        if depth_breakdown is not None:
            record["depth_breakdown"] = depth_breakdown
        if missing_aspects:
            record["missing_aspects"] = missing_aspects

        with LOG_FILE.open("a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")

        _rotate_if_needed()
    except Exception:
        # Telemetry write failures must never reach the user.
        pass


def _rotate_if_needed() -> None:
    """Keep the file bounded by streaming tail + atomic replace.

    Uses `deque(f, MAX_LOG_LINES)` so we never hold more than
    MAX_LOG_LINES strings in memory, regardless of file size.
    """
    try:
        if not LOG_FILE.exists():
            return
        original_size = LOG_FILE.stat().st_size
        if original_size < ROTATE_BYTES:
            return
        # Stream and keep only the last MAX_LOG_LINES.
        with LOG_FILE.open("r", encoding="utf-8") as f:
            tail = list(deque(f, MAX_LOG_LINES))
        # Compute the bytes the rotated file would actually contain. If
        # the tail is the entire file (very few but very long lines that
        # together exceed ROTATE_BYTES), rotation would not shrink
        # anything — skip the write to avoid pointless I/O and a
        # spurious atomic-replace tick. This replaces the previous
        # double-stat check, which compared the same value twice.
        tail_bytes = sum(len(line.encode("utf-8")) for line in tail)
        if tail_bytes >= original_size:
            return

        fd, tmp_path = tempfile.mkstemp(
            prefix=".adaptive-guard.rot.", dir=str(LOG_DIR)
        )
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as tf:
                tf.writelines(tail)
            os.replace(tmp_path, LOG_FILE)
        except Exception:
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
            raise
    except Exception:
        pass


def log_exception(exception: BaseException) -> None:
    """Record an uncaught exception with just enough context to debug.

    We record the exception class name PLUS the last frame (filename +
    line number). We do NOT record `str(exception)` because exception
    messages can include fragments of parsed data (e.g.
    `KeyError('sk-ant-xyz')`). Frame info is safe and lets a developer
    grep the source when a user reports an error.
    """
    try:
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        tb = exception.__traceback__
        frame_info = ""
        if tb is not None:
            frames = traceback.extract_tb(tb)
            if frames:
                last = frames[-1]
                # Filename + line only. No local variable values.
                frame_info = f" at {os.path.basename(last.filename)}:{last.lineno}"
        with ERR_LOG_FILE.open("a", encoding="utf-8") as f:
            f.write(f"{type(exception).__name__}{frame_info}\n")
    except Exception:
        pass
