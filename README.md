# codexline

codexline is a Codex-oriented statusline toolkit inspired by CCometixLine.

## Core capabilities

- Git integration: branch, dirty state, staged/unstaged/untracked/conflicted counts, ahead/behind.
- Model and usage display from Codex rollout files.
- Context and token usage segments.
- Rate-limit segment support when rollout contains window data.
- Theme system with built-in presets and custom theme files.
- Full interactive TUI configurator.
- Patch mode compatibility diagnostics (no binary patching).

## Quick start

```bash
cargo build --release
cargo run -- --init
cargo run -- --plain
```

## Interactive features

- Main menu mode for interactive operations (`--menu`).
- Full TUI configurator (`--config`) with:
  - Theme selection
  - Segment enable/disable
  - Segment reorder
  - Live preview
  - Save and reset

## Commands

- `codexline`
- `codexline --plain`
- `codexline --json`
- `codexline --config`
- `codexline --menu`
- `codexline --theme gruvbox`
- `codexline --init`
- `codexline --print`
- `codexline --check`
- `codexline --doctor`
- `codexline --inspect all`
- `codexline --patch`
- `codexline --patch --json`

## Configuration

- Config file: `~/.codex/codexline/config.toml`
- Theme directory: `~/.codex/codexline/themes/`
- Codex home: `CODEX_HOME` or `~/.codex`

### config.toml example

```toml
theme = "default"

[style]
mode = "nerd_font" # plain | nerd_font | powerline
separator = " · "

[rollout]
scan_depth_days = 14
max_files = 200
# path_override = "/custom/sessions/path"

[diagnostics]
warn_once = true

[[segments]]
id = "model"
enabled = true

[segments.icon]
plain = "M"
nerd_font = "󰭹"

[segments.colors]
icon = "cyan"
text = "bright_cyan"
background = "black"

[segments.styles]
text_bold = false

[segments.options]
# segment-specific options
```

### Segment options

- `cwd.basename` (bool, default `true`): show only current directory basename.
- `git.detailed` (bool, default `false`): include staged/unstaged/untracked/conflicted counters.
- `context.mode` (`remaining` | `used`, default `remaining`): switch context usage wording.

## Themes

Built-in themes:

- `default`
- `minimal`
- `gruvbox`
- `nord`
- `powerline-dark`
- `powerline-light`
- `powerline-rose-pine`
- `powerline-tokyo-night`

Custom theme format (`~/.codex/codexline/themes/<name>.toml`):

```toml
name = "my-theme"

[style]
mode = "nerd_font"
separator = " ❯ "

[[segments]]
id = "git"

[segments.colors]
icon = "bright_magenta"
text = "bright_magenta"
```

## External bridge (no Codex patch)

Use bridge scripts when you want statusline output around Codex without modifying Codex source:

- Windows install: `pwsh -File scripts/bridge/install-bridge.ps1`
- Linux/macOS install: `bash scripts/bridge/install-bridge.sh`
- Windows uninstall: `pwsh -File scripts/bridge/uninstall-bridge.ps1 -RemoveLoopScript`
- Linux/macOS uninstall: `bash scripts/bridge/uninstall-bridge.sh`

See `scripts/bridge/README.md` for full options.

## npm distribution

The npm package lives in `npm/main` and installs platform binaries from GitHub Releases.

Prepare package version from `Cargo.toml`:

```bash
node npm/scripts/prepare-packages.js
```

Verify release matrix and npm installer asset mapping:

```bash
node npm/scripts/verify-release-assets.js
```

Postinstall supports retries and checksum verification via release asset `codexline-checksums.txt`.
See `npm/main/README.md` for environment variables.

## Release automation

- CI: `.github/workflows/ci.yml`
- Binary release: `.github/workflows/release.yml`
- npm publish: `.github/workflows/npm-publish.yml`

Release artifacts expected by npm installer:

- `codexline-windows-x64.exe`
- `codexline-linux-x64`
- `codexline-linux-arm64`
- `codexline-macos-x64`
- `codexline-macos-arm64`
- `codexline-checksums.txt`

## Verify locally

```bash
cargo fmt --check
cargo test
cargo run -- --doctor
cargo run -- --doctor --json
cargo run -- --patch --json
node npm/scripts/verify-release-assets.js
```
