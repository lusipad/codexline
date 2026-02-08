# codexline bridge scripts

These scripts install an external statusline bridge without modifying Codex itself.

## Files

- `install-bridge.ps1`: Windows installer (Task Scheduler + loop script)
- `uninstall-bridge.ps1`: Windows uninstaller
- `install-bridge.sh`: Linux/macOS installer (`systemd --user` or `launchd`)
- `uninstall-bridge.sh`: Linux/macOS uninstaller

## Windows

Install:

```powershell
pwsh -File scripts/bridge/install-bridge.ps1
```

Uninstall:

```powershell
pwsh -File scripts/bridge/uninstall-bridge.ps1 -RemoveLoopScript
```

## Linux / macOS

Install:

```bash
bash scripts/bridge/install-bridge.sh
```

Uninstall:

```bash
bash scripts/bridge/uninstall-bridge.sh
```

## Notes

- The bridge writes a cache line to:
  - Windows: `%LOCALAPPDATA%/codexline/line.txt`
  - Linux/macOS: `${XDG_CACHE_HOME:-~/.cache}/codexline/line.txt`
- Add your shell prompt hook manually using the snippet printed by the installer.
