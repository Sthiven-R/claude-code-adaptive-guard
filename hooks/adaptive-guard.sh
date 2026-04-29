#!/usr/bin/env bash
# adaptive-guard — Claude Code Stop hook entry point.
#
# Reads hook input JSON from stdin, delegates analysis to analyze.py.
#
# Protocol:
#   stdin:  JSON with session_id, stop_hook_active, transcript_path
#   stdout: (empty on success)
#   stderr: blocking reason (only on exit 2)
#   exit 0: allow stop (response is acceptable)
#   exit 2: block stop (force Claude to reconsider with more depth)
#
# Anti-loop: if stop_hook_active=true, exits 0 immediately.
# Anti-crash: if analyze.py fails, exits 0 (fail-open — never block
#             the user from stopping due to guard errors).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANALYZER="$SCRIPT_DIR/lib/analyze.py"
CONFIG_DIR="$SCRIPT_DIR/../config"
DEFAULT_CONFIG="$CONFIG_DIR/default.json"

# Resolve config: env override > default
CONFIG_PATH="${ADAPTIVE_GUARD_CONFIG:-$DEFAULT_CONFIG}"

# Resolve python: must be a real interpreter, not a Windows store
# launcher. Test with noop import to verify.
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
  # No python available — fail-open silently.
  # The guard is a quality tool, not a blocker. If it can't run, allow stop.
  exit 0
fi

if [ ! -f "$ANALYZER" ]; then
  # Misconfigured install — fail-open.
  exit 0
fi

# Read entire stdin (hook payload from Claude Code)
HOOK_INPUT="$(cat)"

# Delegate to analyzer. analyzer decides exit code (0 or 2).
# Pass config path as first arg, hook JSON via stdin.
echo "$HOOK_INPUT" | "$PYTHON_BIN" "$ANALYZER" "$CONFIG_PATH"
exit $?
