#!/usr/bin/env bash
# uninstall.sh — Removes adaptive-guard from ~/.claude/settings.json.
#
# Usage:
#   ./scripts/uninstall.sh [--settings-path <path>]   # for testing on a copy

set -euo pipefail

SETTINGS="$HOME/.claude/settings.json"

while [ $# -gt 0 ]; do
  case "$1" in
    --settings-path)
      SETTINGS="$2"
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

if [ ! -f "$SETTINGS" ]; then
  echo "No settings.json found at $SETTINGS — nothing to remove."
  exit 0
fi

TS="$(date +%Y%m%d-%H%M%S)"
BACKUP="${SETTINGS}.backup-${TS}"
cp "$SETTINGS" "$BACKUP"
echo "Backup created: $BACKUP"

# Resolve working python (same logic as install.sh)
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
  echo "Error: a working Python 3 interpreter was not found." >&2
  exit 1
fi

"$PYTHON_BIN" - "$SETTINGS" <<'PY'
import json
import sys
from pathlib import Path

p = Path(sys.argv[1])
s = json.loads(p.read_text(encoding="utf-8"))

changed = False

stop_hooks = s.get("hooks", {}).get("Stop", [])
new_stop = []

def shell_tokenize(cmd: str) -> list:
    """Split a shell command respecting single-quoted segments.

    Without this, `cmd.split()` breaks paths-with-spaces apart even when
    they were single-quoted by our installer. Real-world example:
        '/c/Program Files/bash' '/c/Users/Foo Bar/repo/adaptive-guard.sh'
    naive split → 4 tokens; respecting quotes → 2 tokens.
    """
    tokens, current, in_squote = [], [], False
    for ch in cmd:
        if ch == "'":
            in_squote = not in_squote
        elif ch in (" ", "\t") and not in_squote:
            if current:
                tokens.append("".join(current))
                current = []
        else:
            current.append(ch)
    if current:
        tokens.append("".join(current))
    return tokens


def is_our_hook(h):
    """Strict match: the hook is ours ONLY if:
      (a) id == "adaptive-guard"  (current installer marker), OR
      (b) command is the simple form `<bash> <adaptive-guard.sh path>`
          with no additional chaining (no &&, ||, ;, |).
    Never match by substring alone — a user-composed command like
    `bash adaptive-guard.sh && bash other.sh` must NOT be removed.
    """
    if h.get("id") == "adaptive-guard":
        return True
    cmd = h.get("command", "")
    if not isinstance(cmd, str):
        return False
    if "adaptive-guard.sh" not in cmd:
        return False
    if any(tok in cmd for tok in ["&&", "||", ";", "|", "$(", "`"]):
        return False
    parts = shell_tokenize(cmd)
    return len(parts) == 2 and parts[1].endswith("adaptive-guard.sh")

for group in stop_hooks:
    filtered = [h for h in group.get("hooks", []) if not is_our_hook(h)]
    if filtered:
        group["hooks"] = filtered
        new_stop.append(group)
    else:
        changed = True

if stop_hooks != new_stop:
    s.setdefault("hooks", {})["Stop"] = new_stop
    changed = True

env = s.get("env", {})
if "ADAPTIVE_GUARD_CONFIG" in env:
    del env["ADAPTIVE_GUARD_CONFIG"]
    changed = True

if changed:
    p.write_text(json.dumps(s, indent=2) + "\n", encoding="utf-8")
    print("adaptive-guard removed from settings.json")
else:
    print("adaptive-guard not found in settings.json — no changes.")
PY
