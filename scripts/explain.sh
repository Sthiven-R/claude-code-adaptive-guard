#!/usr/bin/env bash
# explain.sh - Score any prompt and response interactively to see exactly
# how the guard would decide.
#
# Usage (interactive):
#   ./scripts/explain.sh
#     (prompts you to paste the prompt, then the response)
#
# Usage (from files):
#   ./scripts/explain.sh --prompt prompt.txt --response response.txt
#
# Usage (pipe):
#   cat prompt.txt | ./scripts/explain.sh --stdin-prompt
#     (then asked to paste the response)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_FILE="$REPO_ROOT/config/default.json"

PROMPT_FILE=""
RESPONSE_FILE=""
STDIN_PROMPT="0"

while [ $# -gt 0 ]; do
  case "$1" in
    --prompt)
      PROMPT_FILE="$2"
      shift 2
      ;;
    --response)
      RESPONSE_FILE="$2"
      shift 2
      ;;
    --config)
      CONFIG_FILE="$2"
      shift 2
      ;;
    --stdin-prompt)
      STDIN_PROMPT="1"
      shift
      ;;
    -h|--help)
      grep "^#" "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 1
      ;;
  esac
done

PYTHON_BIN=""
for candidate in python3 python; do
  if command -v "$candidate" >/dev/null 2>&1; then
    if "$candidate" -c "import sys; sys.exit(0)" >/dev/null 2>&1; then
      PYTHON_BIN="$candidate"
      break
    fi
  fi
done
if [ -z "$PYTHON_BIN" ]; then
  echo "Error: no working Python interpreter found." >&2
  exit 1
fi

# Collect prompt text
if [ -n "$PROMPT_FILE" ]; then
  PROMPT_TEXT="$(cat "$PROMPT_FILE")"
elif [ "$STDIN_PROMPT" = "1" ]; then
  PROMPT_TEXT="$(cat)"
else
  echo "Paste the PROMPT text. End with Ctrl-D on a new line:" >&2
  PROMPT_TEXT="$(cat)"
fi

# Collect response text
if [ -n "$RESPONSE_FILE" ]; then
  RESPONSE_TEXT="$(cat "$RESPONSE_FILE")"
else
  echo "" >&2
  echo "Paste the RESPONSE text (leave empty to skip depth scoring). End with Ctrl-D:" >&2
  RESPONSE_TEXT="$(cat)"
fi

PROMPT_TEXT="$PROMPT_TEXT" RESPONSE_TEXT="$RESPONSE_TEXT" CONFIG_FILE="$CONFIG_FILE" \
  "$PYTHON_BIN" - <<'PY'
import json
import os
import sys

repo_root = os.environ.get("REPO_ROOT") or os.path.dirname(
    os.path.dirname(os.path.abspath(sys.argv[0] if sys.argv[0] else "."))
)
# Resolve repo root via the config file's location
config_path = os.environ["CONFIG_FILE"]
repo_root = os.path.dirname(os.path.dirname(config_path))
sys.path.insert(0, os.path.join(repo_root, "hooks", "lib"))

from complexity import score_complexity_explained  # noqa: E402
from depth import score_depth_explained, detect_missing_aspects  # noqa: E402
from config import load_config  # noqa: E402

cfg = load_config(config_path)
thresholds = cfg.get("thresholds", {})
c_thresh = thresholds.get("complexity_min_score", 40)
d_thresh = thresholds.get("depth_min_score", 40)

prompt = os.environ["PROMPT_TEXT"]
response = os.environ.get("RESPONSE_TEXT", "")

def print_breakdown(label, score, breakdown, threshold):
    print()
    print(f"=== {label} TOTAL: {score} / 100  (threshold: {threshold}) ===")
    if breakdown.get("structural") is not None:
        print(f"  structural: {breakdown['structural']}")
    if breakdown.get("semantic") is not None:
        print(f"  semantic:   {breakdown['semantic']}")
        w = breakdown.get("blend_weights")
        if w:
            print(f"  blend:      {w['semantic']:.2f} semantic + {w['structural']:.2f} structural")
    axes = breakdown.get("axes", {})
    if axes:
        print("  Axes:")
        for axis, pts in axes.items():
            print(f"    {axis:<20} {pts:>3} pts")
    signals = breakdown.get("signals", {})
    if signals:
        print("  Signals detected:")
        for k, v in signals.items():
            if k == "tech_tokens":
                any_tokens = {kk: vv for kk, vv in v.items() if vv}
                if any_tokens:
                    for tk, tv in any_tokens.items():
                        print(f"    tech.{tk}: {tv}")
            else:
                print(f"    {k}: {v}")

print()
print("=" * 72)
print("ADAPTIVE-GUARD EXPLAIN")
print("=" * 72)
print(f"Profile:              {cfg.get('profile', 'default')}")
print(f"Complexity threshold: {c_thresh}")
print(f"Depth threshold:      {d_thresh}")
print(f"Prompt length:        {len(prompt)} chars, {len(prompt.split())} words")
if response.strip():
    print(f"Response length:      {len(response)} chars, {len(response.split())} words")

c_score, c_breakdown = score_complexity_explained(prompt)
print_breakdown("COMPLEXITY", c_score, c_breakdown, c_thresh)

if c_score < c_thresh:
    print()
    print(">>> DECISION: allow_simple_task")
    print(f"    Reason:   complexity {c_score} is below threshold {c_thresh}")
    print(f"    The response would NOT be evaluated at all.")
    sys.exit(0)

if not response.strip():
    print()
    print(">>> Response not provided. Cannot simulate full decision.")
    sys.exit(0)

d_score, d_breakdown = score_depth_explained(response)
print_breakdown("DEPTH", d_score, d_breakdown, d_thresh)

if d_score >= d_thresh:
    print()
    print(">>> DECISION: allow_deep_response")
    print(f"    Reason:   depth {d_score} meets threshold {d_thresh}")
else:
    missing = detect_missing_aspects(prompt, response, cfg)
    print()
    print(">>> DECISION: block")
    print(f"    Reason:   depth {d_score} below threshold {d_thresh}")
    if missing:
        print(f"    Missing aspects ({len(missing)}):")
        for m in missing:
            print(f"      - {m}")
    else:
        print("    Missing aspects: (general depth insufficient)")

print()
PY
