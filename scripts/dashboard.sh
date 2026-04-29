#!/usr/bin/env bash
# dashboard.sh — launch the Adaptive Guard desktop dashboard.
#
# Resolution order:
#   1. The release-built binary, if it exists at the standard Tauri
#      output path. Fastest startup, no Node required at runtime.
#   2. Dev mode (`npm run tauri dev`), if `dashboard/node_modules` is
#      installed. Slower startup but always works for developers.
#   3. Print actionable instructions to install / build, then exit.
#
# Usage:
#   adaptive-guard dashboard         # launch in foreground
#   adaptive-guard dashboard &       # background, returns immediately
#
# Note: most users install via the .msi / .dmg / .AppImage from a
# release. They do NOT need this script — they launch from their start
# menu / dock / app launcher. This script is for developers running
# from a cloned repo, and for power users who prefer a single shell
# entry point for everything.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DASHBOARD_DIR="$REPO_ROOT/dashboard"
TAURI_DIR="$DASHBOARD_DIR/src-tauri"

if [ ! -d "$DASHBOARD_DIR" ]; then
  echo "Error: dashboard directory not found at $DASHBOARD_DIR" >&2
  echo "The repo layout is unexpected. Re-clone or check your install." >&2
  exit 1
fi

# 1. Look for the release binary. Tauri's bundle path differs slightly
# per OS; we just probe the common locations.
case "$(uname -s)" in
  Darwin*)
    APP_BIN="$TAURI_DIR/target/release/bundle/macos/Adaptive Guard.app/Contents/MacOS/Adaptive Guard"
    UNIVERSAL_BIN="$TAURI_DIR/target/universal-apple-darwin/release/bundle/macos/Adaptive Guard.app/Contents/MacOS/Adaptive Guard"
    if [ -x "$UNIVERSAL_BIN" ]; then
      exec "$UNIVERSAL_BIN" "$@"
    elif [ -x "$APP_BIN" ]; then
      exec "$APP_BIN" "$@"
    fi
    ;;
  Linux*)
    BIN="$TAURI_DIR/target/release/adaptive-guard-dashboard"
    if [ -x "$BIN" ]; then
      exec "$BIN" "$@"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    BIN="$TAURI_DIR/target/release/adaptive-guard-dashboard.exe"
    if [ -x "$BIN" ]; then
      exec "$BIN" "$@"
    fi
    ;;
esac

# 2. Fall back to dev mode if node_modules are present.
if [ -d "$DASHBOARD_DIR/node_modules" ]; then
  echo "Release build not found; launching in dev mode (this is slower)."
  echo "To build a fast release binary instead:"
  echo "    cd \"$DASHBOARD_DIR\" && npm run tauri build"
  echo ""
  cd "$DASHBOARD_DIR"
  exec npm run tauri dev
fi

# 3. Nothing usable is installed — guide the user.
cat >&2 <<EOF
Error: cannot launch the dashboard. Neither a release build nor a dev
environment was found.

To fix this, choose ONE of the following:

  Option A — install the release bundle (recommended for end users):
    Download the latest .msi / .dmg / .AppImage from
    https://github.com/Sthiven-R/claude-code-adaptive-guard/releases
    and run it. After install, just open "Adaptive Guard" from your
    start menu / Applications. You do NOT need this command.

  Option B — set up dev mode (for contributors):
    cd "$DASHBOARD_DIR"
    npm install
    npm run tauri dev

  Option C — build a release binary locally (slow first time):
    cd "$DASHBOARD_DIR"
    npm install
    npm run tauri build
    Then re-run \`adaptive-guard dashboard\`.

EOF
exit 1
