#!/usr/bin/env bash
# stats.sh - Human-readable summary and inspection of adaptive-guard telemetry.
#
# Usage:
#   ./scripts/stats.sh                # summary of all decisions
#   ./scripts/stats.sh --recent 20    # only the last N decisions
#   ./scripts/stats.sh --today        # only decisions from today
#   ./scripts/stats.sh --last         # full breakdown of the last decision
#   ./scripts/stats.sh --last 5       # full breakdown of the last 5 decisions
#   ./scripts/stats.sh --session <id> # all decisions from a specific session

set -euo pipefail

TELEMETRY="$HOME/.claude/telemetry/adaptive-guard.jsonl"
ERR_LOG="$HOME/.claude/telemetry/adaptive-guard.err.log"

LIMIT=""
TODAY_ONLY="0"
LAST_DETAIL=""
SESSION_FILTER=""

while [ $# -gt 0 ]; do
  case "$1" in
    --recent)
      LIMIT="$2"
      shift 2
      ;;
    --today)
      TODAY_ONLY="1"
      shift
      ;;
    --last)
      # Optional numeric arg; defaults to 1
      if [ $# -ge 2 ] && [[ "$2" =~ ^[0-9]+$ ]]; then
        LAST_DETAIL="$2"
        shift 2
      else
        LAST_DETAIL="1"
        shift
      fi
      ;;
    --session)
      SESSION_FILTER="$2"
      shift 2
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

if [ ! -f "$TELEMETRY" ]; then
  echo "No telemetry yet. The guard hasn't logged any decisions."
  echo "(expected location: $TELEMETRY)"
  exit 0
fi

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
  echo "Error: no working Python interpreter found."
  exit 1
fi

LIMIT="$LIMIT" TODAY_ONLY="$TODAY_ONLY" LAST_DETAIL="$LAST_DETAIL" \
  SESSION_FILTER="$SESSION_FILTER" \
  "$PYTHON_BIN" - "$TELEMETRY" "$ERR_LOG" <<'PY'
import json
import os
import sys
from collections import Counter
from datetime import date

path = sys.argv[1]
err_path = sys.argv[2]
limit = os.environ.get("LIMIT", "")
today_only = os.environ.get("TODAY_ONLY") == "1"
last_detail = os.environ.get("LAST_DETAIL", "")
session_filter = os.environ.get("SESSION_FILTER", "")

try:
    limit_n = int(limit) if limit else None
except ValueError:
    limit_n = None

try:
    last_n = int(last_detail) if last_detail else 0
except ValueError:
    last_n = 0

today_str = date.today().isoformat()

records = []
with open(path, "r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            r = json.loads(line)
        except json.JSONDecodeError:
            continue
        ts = r.get("ts", "")
        if today_only and not ts.startswith(today_str):
            continue
        if session_filter and not r.get("session_id", "").startswith(session_filter):
            continue
        records.append(r)

if limit_n:
    records = records[-limit_n:]

total = len(records)
if total == 0:
    print("No matching records.")
    sys.exit(0)


# ---------------- DETAIL MODE (--last) ----------------
if last_n > 0:
    detailed = records[-last_n:]

    def fmt_decision(dec):
        return {
            "block": "[BLOCK]",
            "allow_deep_response": "[ALLOW DEEP]",
            "allow_simple_task": "[ALLOW SIMPLE]",
        }.get(dec, "[?]")

    for idx, r in enumerate(detailed, 1):
        print()
        print("=" * 72)
        print(f"DECISION #{idx} OF LAST {last_n}")
        print("=" * 72)
        print(f"  Timestamp:    {r.get('ts')}")
        print(f"  Session ID:   {r.get('session_id')}")
        print(f"  Profile:      {r.get('profile')}")
        print(f"  Decision:     {fmt_decision(r.get('decision',''))} {r.get('decision')}")
        print(f"  Prompt chars: {r.get('prompt_chars', '?')}")
        print(f"  Response chars: {r.get('response_chars', '?')}")
        thresholds = r.get("thresholds", {})
        if thresholds:
            print(f"  Thresholds in effect:")
            print(f"    complexity_min_score = {thresholds.get('complexity_min_score')}")
            print(f"    depth_min_score      = {thresholds.get('depth_min_score')}")

        # Complexity breakdown
        cb = r.get("complexity_breakdown")
        if cb:
            print()
            print(f"  COMPLEXITY TOTAL: {r.get('complexity')} / 100")
            if "structural" in cb and cb["structural"] is not None:
                print(f"    structural score:  {cb['structural']}")
            if "semantic" in cb and cb["semantic"] is not None:
                print(f"    semantic score:    {cb['semantic']}")
                w = cb.get("blend_weights")
                if w:
                    print(f"    blend:             {w['semantic']:.2f} semantic + {w['structural']:.2f} structural")
            if cb.get("axes"):
                print(f"    Axes:")
                for axis, pts in cb["axes"].items():
                    print(f"      {axis:<20} {pts:>3} pts")
            if cb.get("signals"):
                sig = cb["signals"]
                print(f"    Signals detected:")
                for k, v in sig.items():
                    # tech_token_counts is a {name: count} dict — render
                    # each non-zero entry as a "tech.<name>: <count>" row.
                    # The pre-2026-05 schema used "tech_tokens"; renaming
                    # the consumer side rather than the producer keeps
                    # historical telemetry readable.
                    if k == "tech_token_counts":
                        any_tokens = {kk: vv for kk, vv in v.items() if vv}
                        if any_tokens:
                            for tk, tv in any_tokens.items():
                                print(f"      tech.{tk}: {tv}")
                    else:
                        print(f"      {k}: {v}")
        else:
            print(f"  COMPLEXITY: {r.get('complexity')} (no breakdown recorded)")

        # Depth breakdown
        db = r.get("depth_breakdown")
        depth = r.get("depth")
        if depth is None:
            print()
            print(f"  DEPTH: not evaluated (complexity below threshold)")
        elif db:
            print()
            print(f"  DEPTH TOTAL: {depth} / 100")
            if "structural" in db and db["structural"] is not None:
                print(f"    structural score: {db['structural']}")
            if "semantic" in db and db["semantic"] is not None:
                print(f"    semantic score:   {db['semantic']}")
            if db.get("axes"):
                print(f"    Axes:")
                for axis, pts in db["axes"].items():
                    print(f"      {axis:<20} {pts:>3} pts")
            if db.get("signals"):
                print(f"    Signals detected:")
                for k, v in db["signals"].items():
                    print(f"      {k}: {v}")
        else:
            print(f"  DEPTH: {depth} (no breakdown recorded)")

        # Missing aspects (on block)
        missing = r.get("missing_aspects")
        if missing:
            print()
            print(f"  MISSING ASPECTS detected ({r.get('missing_count', len(missing))}):")
            for m in missing:
                print(f"    - {m}")

    print()
    sys.exit(0)


# ---------------- SUMMARY MODE ----------------
counter = Counter(r.get("decision", "?") for r in records)
blocks = [r for r in records if r.get("decision") == "block"]
deep = [r for r in records if r.get("decision") == "allow_deep_response"]

def avg(values):
    vals = [v for v in values if isinstance(v, (int, float))]
    return sum(vals) / len(vals) if vals else 0

print()
print("  adaptive-guard telemetry")
print("  " + "-" * 50)
print(f"  Total decisions:        {total}")
if today_only:
    print(f"  (filter: today only - {today_str})")
if limit_n:
    print(f"  (filter: last {limit_n})")
if session_filter:
    print(f"  (filter: session startswith {session_filter})")
print()

for label, key in [
    ("Blocks (guard intervened)", "block"),
    ("Allowed - deep response",    "allow_deep_response"),
    ("Allowed - simple task",      "allow_simple_task"),
]:
    count = counter.get(key, 0)
    pct = (count / total * 100) if total else 0
    bar = "#" * int(pct / 2)
    print(f"  {label:<26} {count:>4}  ({pct:>5.1f}%)  {bar}")
print()

if blocks:
    avg_c = avg(b.get("complexity") for b in blocks)
    avg_d = avg(b.get("depth") for b in blocks)
    avg_m = avg(b.get("missing_count", 0) for b in blocks)
    print(f"  When blocked, avg scores:")
    print(f"    complexity: {avg_c:.1f}  (threshold: 40)")
    print(f"    depth:      {avg_d:.1f}  (threshold: 40)")
    print(f"    missing aspects avg: {avg_m:.1f}")
    print()

if deep:
    avg_c = avg(r.get("complexity") for r in deep)
    avg_d = avg(r.get("depth") for r in deep)
    print(f"  When allowed (deep response), avg scores:")
    print(f"    complexity: {avg_c:.1f}")
    print(f"    depth:      {avg_d:.1f}")
    print()

print("  Last 5 decisions:")
for r in records[-5:]:
    ts = r.get("ts", "")[:19].replace("T", " ")
    dec = r.get("decision", "?")
    c = r.get("complexity")
    d = r.get("depth")
    c_str = f"{c}" if c is not None else "-"
    d_str = f"{d}" if d is not None else "-"
    marker = {
        "block": "[BLOCK]",
        "allow_deep_response": "[ALLOW]",
        "allow_simple_task": "[    -]",
    }.get(dec, "[  ?  ]")
    print(f"    {marker}  {ts}  {dec:<22} complexity={c_str:<4} depth={d_str}")
print()

if os.path.exists(err_path):
    with open(err_path, "r", encoding="utf-8") as f:
        errors = [line.strip() for line in f if line.strip()]
    if errors:
        print(f"  Error log: {len(errors)} recorded errors")
        for e in errors[-3:]:
            print(f"    - {e}")
        print()
    else:
        print("  Error log: clean")
        print()
else:
    print("  Error log: clean (no errors ever recorded)")
    print()

print("  TIP: Run with '--last' to see full breakdown of the most recent decision.")
print("       Run with '--last 5' to see breakdowns of the last 5.")
print()
PY
