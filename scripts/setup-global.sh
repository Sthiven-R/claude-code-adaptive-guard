#!/usr/bin/env bash
# setup-global.sh - One-time setup: install 'adaptive-guard' as a global
# command in the user's Unix shell. Safe, idempotent.
#
# Steps:
#   1. Resolve repo root (parent of this script).
#   2. Create ~/bin/ if missing.
#   3. Create ~/.adaptive-guard/config with REPO_ROOT.
#   4. Copy cli/adaptive-guard -> ~/bin/adaptive-guard (chmod +x).
#   5. If ~/bin is NOT on PATH, append it to ~/.bashrc or ~/.zshrc.
#
# Run from the repo root:
#   ./scripts/setup-global.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Setting up adaptive-guard as a global command on this machine."
echo ""
echo "  Repo: $REPO_ROOT"
echo ""

# 1. Bin dir
BIN_DIR="${HOME}/bin"
mkdir -p "$BIN_DIR"

# 2. Config file
CONF_DIR="${HOME}/.adaptive-guard"
mkdir -p "$CONF_DIR"
CONF_FILE="$CONF_DIR/config"
printf 'REPO_ROOT=%s\n' "$REPO_ROOT" > "$CONF_FILE"
echo "Wrote config: $CONF_FILE"

# 3. Copy dispatcher
SRC="$REPO_ROOT/cli/adaptive-guard"
DST="$BIN_DIR/adaptive-guard"
if [ ! -f "$SRC" ]; then
  echo "Error: dispatcher not found at $SRC" >&2
  exit 1
fi
cp "$SRC" "$DST"
chmod +x "$DST"
echo "Installed dispatcher: $DST"

# 4. PATH check
case ":$PATH:" in
  *":$BIN_DIR:"*)
    echo ""
    echo "$BIN_DIR is already on PATH. You can use 'adaptive-guard' right away."
    ;;
  *)
    # Detect shell rc
    SHELL_RC=""
    if [ -n "${ZSH_VERSION:-}" ] || [ "$(basename "${SHELL:-}")" = "zsh" ]; then
      SHELL_RC="${HOME}/.zshrc"
    elif [ "$(basename "${SHELL:-}")" = "bash" ]; then
      [ -f "${HOME}/.bashrc" ] && SHELL_RC="${HOME}/.bashrc" || SHELL_RC="${HOME}/.bash_profile"
    else
      SHELL_RC="${HOME}/.profile"
    fi

    LINE='export PATH="$HOME/bin:$PATH"'
    if [ -f "$SHELL_RC" ] && grep -Fq "$LINE" "$SHELL_RC"; then
      :
    else
      {
        echo ""
        echo "# Added by adaptive-guard setup-global.sh"
        echo "$LINE"
      } >> "$SHELL_RC"
      echo ""
      echo "Appended PATH line to $SHELL_RC"
      echo ""
      echo "  IMPORTANT: open a NEW shell (or run 'source $SHELL_RC') for the change to take effect."
      echo "  Then run:  adaptive-guard help"
    fi
    ;;
esac

echo ""
echo "Setup complete."
echo ""
echo "Try:"
echo "  adaptive-guard help"
echo "  adaptive-guard version"
echo "  adaptive-guard stats --last"
echo ""
