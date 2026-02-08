#!/usr/bin/env bash
set -euo pipefail

REFRESH_MS=800
MODE="auto"
NO_START=0

usage() {
  cat <<'USAGE'
Usage:
  install-bridge.sh [--refresh-ms <ms>] [--mode auto|systemd|launchd|none] [--no-start]

Examples:
  install-bridge.sh
  install-bridge.sh --refresh-ms 500
  install-bridge.sh --mode none --no-start
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --refresh-ms)
      REFRESH_MS="${2:-}"
      shift 2
      ;;
    --mode)
      MODE="${2:-}"
      shift 2
      ;;
    --no-start)
      NO_START=1
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

if ! [[ "$REFRESH_MS" =~ ^[0-9]+$ ]]; then
  echo "--refresh-ms must be an integer." >&2
  exit 1
fi

if (( REFRESH_MS < 200 )); then
  echo "--refresh-ms must be >= 200." >&2
  exit 1
fi

if ! command -v codexline >/dev/null 2>&1; then
  echo "codexline is not in PATH." >&2
  exit 1
fi

SLEEP_SECONDS="$(awk "BEGIN { printf \"%.3f\", ${REFRESH_MS}/1000 }")"
LOOP_PATH="${HOME}/.local/bin/codexline-bridge-loop.sh"
mkdir -p "$(dirname "$LOOP_PATH")"

cat > "$LOOP_PATH" <<EOF_LOOP
#!/usr/bin/env bash
set -u
cache_dir="\${XDG_CACHE_HOME:-\$HOME/.cache}/codexline"
line_path="\$cache_dir/line.txt"
tmp_path="\$cache_dir/line.txt.tmp"
mkdir -p "\$cache_dir"

while true; do
  line="\$(codexline --plain 2>/dev/null || true)"
  if [ -n "\$line" ]; then
    printf "%s" "\$line" > "\$tmp_path" && mv -f "\$tmp_path" "\$line_path"
  fi
  sleep ${SLEEP_SECONDS}
done
EOF_LOOP
chmod +x "$LOOP_PATH"

SERVICE_MODE=""
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

if [[ "$MODE" == "systemd" ]]; then
  SERVICE_FILE="${HOME}/.config/systemd/user/codexline-bridge.service"
  mkdir -p "$(dirname "$SERVICE_FILE")"

  cat > "$SERVICE_FILE" <<EOF_SERVICE
[Unit]
Description=CodexLine Bridge Loop

[Service]
Type=simple
ExecStart=${LOOP_PATH}
Restart=always
RestartSec=2

[Install]
WantedBy=default.target
EOF_SERVICE

  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload
    systemctl --user enable codexline-bridge.service >/dev/null
    if [[ "$NO_START" -eq 0 ]]; then
      systemctl --user restart codexline-bridge.service
    fi
    SERVICE_MODE="systemd"
  else
    echo "systemctl not found. Service file created at: $SERVICE_FILE" >&2
    SERVICE_MODE="systemd-file-only"
  fi
elif [[ "$MODE" == "launchd" ]]; then
  PLIST_PATH="${HOME}/Library/LaunchAgents/com.codexline.bridge.plist"
  mkdir -p "$(dirname "$PLIST_PATH")"

  cat > "$PLIST_PATH" <<EOF_PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.codexline.bridge</string>
    <key>ProgramArguments</key>
    <array>
      <string>${LOOP_PATH}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/codexline-bridge.out</string>
    <key>StandardErrorPath</key>
    <string>/tmp/codexline-bridge.err</string>
  </dict>
</plist>
EOF_PLIST

  if command -v launchctl >/dev/null 2>&1; then
    launchctl bootout "gui/${UID}/com.codexline.bridge" >/dev/null 2>&1 || true
    if [[ "$NO_START" -eq 0 ]]; then
      launchctl bootstrap "gui/${UID}" "$PLIST_PATH"
      launchctl enable "gui/${UID}/com.codexline.bridge"
    fi
    SERVICE_MODE="launchd"
  else
    echo "launchctl not found. Plist created at: $PLIST_PATH" >&2
    SERVICE_MODE="launchd-file-only"
  fi
elif [[ "$MODE" == "none" ]]; then
  if [[ "$NO_START" -eq 0 ]]; then
    nohup "$LOOP_PATH" >/dev/null 2>&1 &
  fi
  SERVICE_MODE="none"
else
  echo "Unsupported mode: $MODE" >&2
  exit 1
fi

echo "[codexline-bridge] Installed."
echo "loop script : $LOOP_PATH"
echo "mode        : $SERVICE_MODE"
echo ""
echo "Prompt snippet (bash/zsh):"
cat <<'SNIPPET'
__codexline_prompt() {
  local f="${XDG_CACHE_HOME:-$HOME/.cache}/codexline/line.txt"
  [ -f "$f" ] || return 0
  local line
  line="$(cat "$f" 2>/dev/null)"
  [ -n "$line" ] && printf "%s\n" "$line"
}
SNIPPET
echo "Use PROMPT_COMMAND (bash) or precmd hook (zsh) to call __codexline_prompt."
