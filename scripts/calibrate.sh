#!/usr/bin/env bash
# calibrate.sh - Cross-reference telemetry with operator feedback to suggest threshold tuning.
#
# Usage:
#   ./scripts/calibrate.sh
#
# Reads ~/.claude/telemetry/adaptive-guard.jsonl (decisions) and
# ~/.claude/telemetry/adaptive-guard-feedback.jsonl (operator's
# useful/annoying labels), joins them, and reports:
#   - How much labeled data exists.
#   - False positive rate (RETRY decisions the operator marked annoying).
#   - False negative rate (PASS / TRIVIAL decisions the operator marked annoying).
#   - Per-axis distribution of "annoying" decisions, to spot systematic bias.
#   - Concrete threshold-adjustment suggestions backed by your own labels.
#
# Honest disclaimer printed at the top of every run: the suggestions
# only get reliable past ~30-50 labels. Below that, they're noise.

set -euo pipefail

TELEMETRY="$HOME/.claude/telemetry/adaptive-guard.jsonl"
FEEDBACK="$HOME/.claude/telemetry/adaptive-guard-feedback.jsonl"

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

"$PYTHON_BIN" - "$TELEMETRY" "$FEEDBACK" <<'PY'
import json
import os
import sys
from collections import defaultdict

telemetry_path = sys.argv[1]
feedback_path = sys.argv[2]


def load_jsonl(path):
    if not os.path.exists(path):
        return []
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return out


print("=" * 72)
print("ADAPTIVE-GUARD CALIBRATE")
print("=" * 72)

records = load_jsonl(telemetry_path)
records_by_key = {(r.get("session_id"), r.get("ts")): r for r in records}

feedback_lines = load_jsonl(feedback_path)
# latest-wins per (session_id, decision_ts)
latest = {}
for fb in feedback_lines:
    key = (fb.get("session_id"), fb.get("decision_ts"))
    if key not in latest or fb.get("ts", "") >= latest[key].get("ts", ""):
        latest[key] = fb
# drop tombstones
active = {k: v for k, v in latest.items() if v.get("label") != "cleared"}

# Match feedback to telemetry records.
matches = []
for key, fb in active.items():
    rec = records_by_key.get(key)
    if rec is None:
        continue
    matches.append((rec, fb))

total_records = len(records)
total_labeled = len(matches)

print(f"Total decisions in telemetry:  {total_records}")
print(f"Decisions with your feedback:  {total_labeled}", end="")
if total_records > 0:
    print(f"  ({100 * total_labeled / total_records:.1f}%)")
else:
    print()

if total_labeled == 0:
    print()
    print("No labeled decisions yet. Open the dashboard, expand a few decision")
    print("cards, and click `Useful` or `Annoying`. Come back here once you")
    print("have at least 20-30 labels for the suggestions to be meaningful.")
    sys.exit(0)

useful = sum(1 for _, fb in matches if fb.get("label") == "useful")
annoying = sum(1 for _, fb in matches if fb.get("label") == "annoying")
print(f"  Useful:   {useful}")
print(f"  Annoying: {annoying}")

if total_labeled < 20:
    print()
    print(f"NOTE: only {total_labeled} labels. Suggestions below are weak signal.")
    print("      Past ~30 labels they become directional; past ~50 they become reliable.")

# ---------------------------------------------------------------------------
# Error rates
# ---------------------------------------------------------------------------
print()
print("Error rates (your feedback as ground truth):")

# A "false positive" is the guard saying RETRY when the operator believes
# the response was actually fine (annoying intervention).
blocks = [(r, fb) for r, fb in matches if r.get("decision") == "block"]
fp = sum(1 for _, fb in blocks if fb.get("label") == "annoying")
n_blocks_labeled = len(blocks)
if n_blocks_labeled:
    print(
        f"  False positives (RETRY but you said annoying):"
        f"  {fp}/{n_blocks_labeled} = {100 * fp / n_blocks_labeled:.1f}%"
    )
else:
    print("  No labeled RETRY decisions yet — false-positive rate unknown.")

# A "false negative" is the guard saying PASS / TRIVIAL when the operator
# believes the response was actually shallow (missed an intervention).
passes = [(r, fb) for r, fb in matches if r.get("decision") != "block"]
fn = sum(1 for _, fb in passes if fb.get("label") == "annoying")
n_passes_labeled = len(passes)
if n_passes_labeled:
    print(
        f"  False negatives (PASS or TRIVIAL but you said annoying):"
        f"  {fn}/{n_passes_labeled} = {100 * fn / n_passes_labeled:.1f}%"
    )
else:
    print("  No labeled PASS / TRIVIAL decisions yet — false-negative rate unknown.")

# ---------------------------------------------------------------------------
# Distribution of complexity / depth scores in annoying decisions
# ---------------------------------------------------------------------------
annoying_records = [r for r, fb in matches if fb.get("label") == "annoying"]
if annoying_records:
    cs = [r.get("complexity") for r in annoying_records if r.get("complexity") is not None]
    ds = [r.get("depth") for r in annoying_records if r.get("depth") is not None]
    print()
    print("Annoying decisions — score distribution:")
    if cs:
        print(
            f"  complexity: min={min(cs)} median={sorted(cs)[len(cs) // 2]} "
            f"max={max(cs)} mean={sum(cs) / len(cs):.1f}"
        )
    if ds:
        print(
            f"  depth:      min={min(ds)} median={sorted(ds)[len(ds) // 2]} "
            f"max={max(ds)} mean={sum(ds) / len(ds):.1f}"
        )

# ---------------------------------------------------------------------------
# Per-axis breakdown — which signals correlate with "annoying"
# ---------------------------------------------------------------------------
axis_pts_in_annoying = defaultdict(list)
for r, fb in matches:
    if fb.get("label") != "annoying":
        continue
    for which in ("complexity_breakdown", "depth_breakdown"):
        bd = r.get(which) or {}
        axes = bd.get("axes") or {}
        for axis, pts in axes.items():
            axis_pts_in_annoying[(which, axis)].append(pts)

if axis_pts_in_annoying:
    print()
    print("Average axis contribution within annoying decisions:")
    rows = []
    for (which, axis), pts in axis_pts_in_annoying.items():
        rows.append((which.replace("_breakdown", ""), axis, sum(pts) / len(pts), len(pts)))
    rows.sort(key=lambda r: (r[0], -r[2]))
    for which, axis, avg, n in rows:
        print(f"  {which:<10} {axis:<20} avg={avg:.1f} (across {n})")

# ---------------------------------------------------------------------------
# Concrete suggestion
# ---------------------------------------------------------------------------
print()
print("Suggestion:")
suggestions = []

if n_blocks_labeled and (fp / n_blocks_labeled) > 0.3:
    suggestions.append(
        f"  - {fp / n_blocks_labeled * 100:.0f}% of your RETRY decisions are 'annoying'. "
        "Consider RAISING `complexity_min_score` (block fewer prompts) or "
        "LOWERING `depth_min_score` (accept slightly less depth)."
    )

if n_passes_labeled and (fn / n_passes_labeled) > 0.3:
    suggestions.append(
        f"  - {fn / n_passes_labeled * 100:.0f}% of your PASS/TRIVIAL decisions are 'annoying'. "
        "Consider LOWERING `complexity_min_score` (evaluate more prompts) or "
        "RAISING `depth_min_score` (demand more depth before passing)."
    )

if not suggestions:
    print("  Nothing actionable yet. Either your error rates are below 30% in")
    print("  both directions (well-calibrated for your judgment), or you need")
    print("  more labeled decisions for a clearer signal.")
else:
    for s in suggestions:
        print(s)
    print()
    print("  Tip: edit the active profile under `config/profiles/` and re-install.")
    print("       The dashboard's gear menu shows the active preset.")
PY
