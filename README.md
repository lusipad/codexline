# codexline

A Codex-oriented statusline CLI inspired by CCometixLine.

## Features

- Independent CLI that does not patch Codex source code
- Real-time environment and Git status collection
- Rollout-based token/context/rate-limit extraction from ~/.codex/sessions
- Graceful degradation when data fields are missing
- Configurable segments via ~/.codex/codexline/config.toml
- Plain text, ANSI, and JSON output modes

## Commands

- codexline
- codexline --plain
- codexline --json
- codexline init
- codexline print-config
- codexline check-config
- codexline doctor
- codexline inspect --source all

## Config Path

- Config file: ~/.codex/codexline/config.toml
- Codex home: CODEX_HOME or ~/.codex
