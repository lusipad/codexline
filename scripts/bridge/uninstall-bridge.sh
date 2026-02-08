#!/usr/bin/env bash
set -euo pipefail

MODE="auto"
REMOVE_LOOP=1

usage() {
  cat <<'USAGE'
Usage:
  uninstall-bridge.sh [--mode auto|systemd|launchd|none] [--keep-loop]

Examples:
  uninstall-bridge.sh
  uninstall-bridge.sh --mode none --keep-loop
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="${2:-}"
      shift 2
      ;;
    --keep-loop)
      REMOVE_LOOP=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

OS_NAME="$(uname -s)"
if [[ "$MODE" == "auto" ]]; then
  if [[ "$OS_NAME" == "Linux" ]]; then
    MODE="systemd"
  elif [[ "$OS_NAME" == "Darwin" ]]; then
    MODE="launchd"
  else
    MODE="none"
  fi
fi

LOOP_PATH="${HOME}/.local/bin/codexline-bridge-loop.sh"

if [[ "$MODE" == "systemd" ]]; then
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop codexline-bridge.service >/dev/null 2>&1 || true
    systemctl --user disable codexline-bridge.service >/dev/null 2>&1 || true
    systemctl --user daemon-reload
  fi
  rm -f "${HOME}/.config/systemd/user/codexline-bridge.service"
elif [[ "$MODE" == "launchd" ]]; then
  if command -v launchctl >/dev/null 2>&1; then
    launchctl bootout "gui/${UID}/com.codexline.bridge" >/dev/null 2>&1 || true
    launchctl disable "gui/${UID}/com.codexline.bridge" >/dev/null 2>&1 || true
  fi
  rm -f "${HOME}/Library/LaunchAgents/com.codexline.bridge.plist"
elif [[ "$MODE" == "none" ]]; then
  :
else
  echo "Unsupported mode: $MODE" >&2
  exit 1
fi

pkill -f "$LOOP_PATH" >/dev/null 2>&1 || true

if [[ "$REMOVE_LOOP" -eq 1 ]]; then
  rm -f "$LOOP_PATH"
fi

echo "[codexline-bridge] Uninstalled."
echo "mode        : $MODE"
echo "loop script : $LOOP_PATH"
if [[ "$REMOVE_LOOP" -eq 0 ]]; then
  echo "loop script kept (--keep-loop)."
fi
echo "If you added prompt hooks manually, remove them from your shell profile."
