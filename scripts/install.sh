#!/usr/bin/env bash
# install.sh — Adds adaptive-guard as a Stop hook in ~/.claude/settings.json.
#
# Safe install:
#   - Backs up existing settings.json with timestamp
#   - Only appends a Stop hook (does not modify anything else)
#   - Idempotent: re-running does nothing if already installed
#
# Usage:
#   ./scripts/install.sh [--profile balanced|strict|lenient]
#                        [--settings-path <path>]   # for testing on a copy

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOK_SCRIPT="$REPO_ROOT/hooks/adaptive-guard.sh"
SETTINGS="$HOME/.claude/settings.json"
PROFILE="balanced"
CONFIG_FILE="$REPO_ROOT/config/default.json"

# Detect bash absolute path. Preference order (Windows Git Bash/MSYS
# use /usr/bin/bash as a stable POSIX path; Claude Code resolves it
# correctly even when spawned from non-bash context):
#   1) /usr/bin/bash  (stable POSIX path on Linux/Mac/MSYS/WSL)
#   2) command -v bash (fallback, may return Windows-style path)
if [ -x /usr/bin/bash ]; then
  BASH_ABS="/usr/bin/bash"
else
  BASH_ABS="$(command -v bash || echo /bin/bash)"
fi

# Convert hook + config paths to POSIX when cygpath is available.
# Python handles all escaping in the injection step below.
to_posix() {
  local p="$1"
  if command -v cygpath >/dev/null 2>&1; then
    p="$(cygpath -u "$p" 2>/dev/null || echo "$p")"
  fi
  printf '%s' "$p"
}

BASH_POSIX="$(to_posix "$BASH_ABS")"
HOOK_POSIX="$(to_posix "$HOOK_SCRIPT")"
CONFIG_POSIX="$(to_posix "$CONFIG_FILE")"

# Parse args
while [ $# -gt 0 ]; do
  case "$1" in
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --config)
      CONFIG_FILE="$2"
      shift 2
      ;;
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

# Resolve profile config
if [ "$PROFILE" != "balanced" ]; then
  PROFILE_FILE="$REPO_ROOT/config/profiles/${PROFILE}.json"
  if [ ! -f "$PROFILE_FILE" ]; then
    echo "Error: profile '$PROFILE' not found at $PROFILE_FILE" >&2
    exit 1
  fi
  CONFIG_FILE="$PROFILE_FILE"
fi

# Preconditions
if [ ! -f "$HOOK_SCRIPT" ]; then
  echo "Error: hook script not found at $HOOK_SCRIPT" >&2
  exit 1
fi

if ! command -v python3 &>/dev/null && ! command -v python &>/dev/null; then
  echo "Error: python3 not found. Adaptive-guard requires Python 3.8+." >&2
  exit 1
fi

chmod +x "$HOOK_SCRIPT" 2>/dev/null || true

# Create settings.json if missing
mkdir -p "$(dirname "$SETTINGS")"
if [ ! -f "$SETTINGS" ]; then
  echo '{}' > "$SETTINGS"
fi

# Backup
TS="$(date +%Y%m%d-%H%M%S)"
BACKUP="${SETTINGS}.backup-${TS}"
cp "$SETTINGS" "$BACKUP"
echo "Backup created: $BACKUP"

# Resolve python binary — must be a real interpreter, not a Windows
# store launcher that just prints "Python was not found". Test with
# a noop import.
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
  echo "Install Python 3.8+ and ensure 'python' or 'python3' is on PATH." >&2
  exit 1
fi

# Inject hook using Python. Python handles all path escaping and
# POSIX normalization, more reliably than bash parameter expansion.
#
# On Windows/MSYS, native python.exe auto-converts POSIX arg paths
# to Windows (C:/Program Files/Git/usr/bin/bash). We accept that here
# — Python needs the settings path in Windows form to open the file —
# and explicitly re-convert to POSIX inside Python for the paths that
# will be embedded in the bash-invoked command string.
"$PYTHON_BIN" - "$SETTINGS" "$BASH_POSIX" "$HOOK_POSIX" "$CONFIG_POSIX" <<'PY'
import json
import re
import sys
from pathlib import Path

settings_path = Path(sys.argv[1])
bash_path = sys.argv[2]
hook_path = sys.argv[3]
config_file = sys.argv[4]


def to_msys_posix(p: str) -> str:
    """Convert a Windows path (C:\\Foo or C:/Foo) to MSYS POSIX (/c/Foo).

    On Linux/macOS, returns the path unchanged.
    Only rewrites the drive-letter prefix; forward-slashes remain.
    """
    # Already POSIX? (starts with / and not a drive letter)
    if p.startswith("/") and not re.match(r"^/[a-zA-Z]:", p):
        return p
    # Windows drive-letter form: C:\ or C:/
    m = re.match(r"^([a-zA-Z]):[/\\](.*)$", p)
    if m:
        drive, rest = m.group(1).lower(), m.group(2).replace("\\", "/")
        return f"/{drive}/{rest}"
    # Unknown form — return as-is (caller will validate)
    return p.replace("\\", "/")


# Paths that will be embedded in the shell-invoked `command` string
# must be in MSYS POSIX form so /usr/bin/bash can resolve them.
bash_path_posix = to_msys_posix(bash_path)
hook_path_posix = to_msys_posix(hook_path)
config_file_posix = to_msys_posix(config_file)

import re

# Inside POSIX single quotes every byte except `'` is literal. The
# `posix_single_quote` helper escapes `'` via close-escape-reopen, so
# shell metacharacters like `$ & ( ) [ ] { }` are SAFE inside our
# quoted paths. The list below rejects only bytes that break path
# validity on real filesystems (newlines, NUL) or signal a path
# conversion bug (a stray backslash after to_msys_posix should have
# stripped them). Mirrors the Rust `DANGEROUS` set in install.rs —
# keep both in sync.
_DANGEROUS_CHARS = set('\n\r\0\\')

def assert_safe_path(p: str, label: str) -> None:
    bad = [c for c in p if c in _DANGEROUS_CHARS]
    if bad:
        sys.stderr.write(
            f"Error: {label} contains characters invalid in a POSIX path "
            f"or signaling a conversion bug ({sorted(set(bad))!r}): {p!r}\n"
            f"A backslash here usually means MSYS conversion did not run; "
            f"newlines or NULs cannot be valid in any path.\n"
        )
        sys.exit(2)

def posix_single_quote(p: str) -> str:
    """Single-quote a POSIX path for embedding in a shell command string.

    Uses the standard close-quote-escape-reopen idiom:
      foo'bar  ->  'foo'\''bar'
    Safe against all POSIX shell metacharacters, including spaces.
    """
    return "'" + p.replace("'", "'\\''") + "'"

assert_safe_path(bash_path_posix, "bash path")
assert_safe_path(hook_path_posix, "hook script path")
# config path is not interpolated into a shell command, only into JSON,
# so it just needs to be valid JSON (strings always are after json.dumps).

command = f"{posix_single_quote(bash_path_posix)} {posix_single_quote(hook_path_posix)}"

try:
    settings = json.loads(settings_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as e:
    sys.stderr.write(
        f"Error: settings.json at {settings_path} is not valid JSON.\n"
        f"  Parse error: {e}\n"
        f"  A backup was already created. Restore it or fix the JSON manually.\n"
    )
    sys.exit(3)

if not isinstance(settings, dict):
    sys.stderr.write("Error: settings.json must be a JSON object at the top level.\n")
    sys.exit(3)

hooks = settings.setdefault("hooks", {})
stop_hooks = hooks.setdefault("Stop", [])

hook_id = "adaptive-guard"


def shell_tokenize(cmd: str) -> list:
    """Split a shell command respecting single-quoted segments.

    Our installer always emits arguments single-quoted; assert_safe rules
    out single quotes inside paths. So a tokenizer that toggles a quote
    flag is enough to recover the original two-arg form even when paths
    contain spaces — e.g. `'/c/Program Files/bash' '/repo/x.sh'` should
    yield two tokens, not four.
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


def is_our_hook(h) -> bool:
    if h.get("id") == hook_id:
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
    for h in group.get("hooks", []):
        if is_our_hook(h):
            print("Already installed -- no changes.")
            sys.exit(0)

stop_hooks.append({
    "hooks": [
        {
            "id": hook_id,
            "type": "command",
            "command": command,
            "timeout": 30,
        }
    ]
})

env = settings.setdefault("env", {})
env["ADAPTIVE_GUARD_CONFIG"] = config_file_posix

settings_path.write_text(json.dumps(settings, indent=2) + "\n", encoding="utf-8")
print("Installed adaptive-guard hook")
print(f"  command: {command}")
print(f"  config:  {config_file_posix}")
PY

echo ""
echo "Done. Restart Claude Code for the hook to take effect."
echo "To uninstall later: $SCRIPT_DIR/uninstall.sh"
