"""
analyze.py - Stop-hook orchestrator for adaptive-guard.

Reads the hook payload (JSON on stdin), picks up the assistant response
from the hook input (with a transcript fallback), recovers the matching
user prompt from the transcript, scores both, and decides whether to
allow the stop or emit a `block` with a specific reason.

This file is kept small and focused on orchestration. The supporting
modules are:

  config.py      -> load_config (extends resolution + deep merge)
  transcript.py  -> bounded transcript read, extract user / assistant
  output.py      -> emit_block_decision, build_block_reason
  complexity.py  -> score_complexity_explained (+ structural fallback)
  depth.py       -> score_depth_explained, detect_missing_aspects
  telemetry.py   -> log_decision, log_exception

Protocol (modern Stop-hook contract):
  exit 0 + empty stdout               -> allow stop
  exit 0 + JSON {decision: "block"}   -> force Claude to continue

Fail-open: any unexpected error -> exit 0 with empty stdout. The guard
is a quality tool; it must never block the user from finishing a
session because of its own bugs.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

# Make sibling modules importable regardless of cwd.
sys.path.insert(0, str(Path(__file__).parent))

# Each helper imports from its canonical home — this orchestrator does
# not re-export. Tests and `scripts/explain.sh` import from the same
# canonical modules, so the dependency graph reflects reality (no false
# dependents on analyze).
from config import load_config  # noqa: E402
from transcript import (  # noqa: E402
    extract_last_assistant_text,
    extract_last_user_prompt,
)
from output import build_block_reason, emit_block_decision  # noqa: E402
from complexity import score_complexity_explained  # noqa: E402
from depth import detect_missing_aspects, score_depth_explained  # noqa: E402
from telemetry import log_decision, log_exception  # noqa: E402


def main() -> int:
    try:
        config_path = sys.argv[1] if len(sys.argv) > 1 else None
        if not config_path:
            return 0

        # Cap stdin to 64 MB. The hook payload is at most a few KB in
        # practice; a runaway feed (process bug upstream, accidental
        # binary on stdin) is bounded so we never OOM. MemoryError is
        # not caught by the outer `except Exception`, so failing here
        # would defeat fail-open.
        hook_input_raw = sys.stdin.read(64 * 1024 * 1024)
        if not hook_input_raw.strip():
            return 0

        try:
            hook_input = json.loads(hook_input_raw)
        except json.JSONDecodeError:
            return 0

        # Anti-loop: if we are already inside a forced-continuation
        # triggered by a prior block, allow the stop to end the chain.
        if hook_input.get("stop_hook_active"):
            return 0

        cfg = load_config(config_path)

        # Primary: the Stop hook input already carries the assistant text.
        assistant_response = hook_input.get("last_assistant_message", "") or ""

        transcript_path = hook_input.get("transcript_path", "") or ""

        # Fallback: parse the transcript if the hook input lacks the response.
        if not assistant_response and transcript_path:
            assistant_response = extract_last_assistant_text(transcript_path)

        if not assistant_response.strip():
            return 0

        # User prompt is only available via transcript.
        user_prompt = ""
        if transcript_path:
            user_prompt = extract_last_user_prompt(transcript_path)

        if not user_prompt.strip():
            return 0

        prompt_chars = len(user_prompt)
        response_chars = len(assistant_response)

        # Stage 1: is the prompt complex enough to warrant evaluating depth?
        complexity, c_breakdown = score_complexity_explained(user_prompt)
        if complexity < cfg["thresholds"]["complexity_min_score"]:
            log_decision(
                cfg, hook_input,
                complexity=complexity, depth=None,
                decision="allow_simple_task",
                complexity_breakdown=c_breakdown,
                prompt_chars=prompt_chars,
                response_chars=response_chars,
            )
            return 0

        # Stage 2: was the response deep enough?
        depth, d_breakdown = score_depth_explained(assistant_response)
        if depth >= cfg["thresholds"]["depth_min_score"]:
            log_decision(
                cfg, hook_input,
                complexity=complexity, depth=depth,
                decision="allow_deep_response",
                complexity_breakdown=c_breakdown,
                depth_breakdown=d_breakdown,
                prompt_chars=prompt_chars,
                response_chars=response_chars,
            )
            return 0

        # Stage 3: block, and tell Claude which aspects look missing.
        missing = detect_missing_aspects(user_prompt, assistant_response, cfg)
        reason = build_block_reason(cfg["blocking_message_template"], missing)

        log_decision(
            cfg, hook_input,
            complexity=complexity, depth=depth,
            decision="block",
            missing_count=len(missing),
            complexity_breakdown=c_breakdown,
            depth_breakdown=d_breakdown,
            missing_aspects=missing,
            prompt_chars=prompt_chars,
            response_chars=response_chars,
        )

        emit_block_decision(reason)
        return 0

    except Exception as e:
        log_exception(e)
        return 0


if __name__ == "__main__":
    sys.exit(main())
