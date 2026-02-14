# codexline

Binary wrapper package for codexline.

This package downloads the correct platform binary from GitHub Releases during postinstall and exposes the `codexline` command.

## Install

```bash
npm install -g codexline
```

One-shot usage without global install:

```bash
npx codexline
```

Behavior with no arguments:

- TTY terminal: opens interactive menu.
- Non-TTY context (pipe/CI): falls back to `--plain`.

Bridge via the same entrypoint:

```bash
npx codexline bridge install
```

## Bridge command

The package also includes `codexline-bridge` for external statusline bridge setup (no Codex core patch):

```bash
codexline-bridge install
codexline-bridge uninstall
```

One-shot `npx` usage:

```bash
npx --package codexline codexline-bridge install
```

The CLI executes bundled platform scripts under `bridge/`:

- Windows: `install-bridge.ps1` / `uninstall-bridge.ps1`
- Linux/macOS: `install-bridge.sh` / `uninstall-bridge.sh`

Pass script options directly after the subcommand, for example:

```bash
codexline-bridge install --refresh-ms 500
```

On Windows, common long options are translated automatically to PowerShell parameter names.

## Environment variables

- `CODEXLINE_SKIP_DOWNLOAD=1`: skip postinstall download.
- `CODEXLINE_VERSION`: override download version.
- `CODEXLINE_BASE_URL`: override release download URL base (default ends with `/v`).
- `CODEXLINE_DOWNLOAD_RETRIES`: max download retry count (default: `3`).
- `CODEXLINE_DOWNLOAD_TIMEOUT_MS`: HTTP timeout in ms (default: `20000`).
- `CODEXLINE_VERIFY_CHECKSUM=0`: disable checksum verification.
- `CODEXLINE_REQUIRE_CHECKSUM=1`: fail install if checksum file/entry is missing.
