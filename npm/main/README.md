# codexline

Binary wrapper package for codexline.

This package downloads the correct platform binary from GitHub Releases during postinstall and exposes the codexline command.

## Install

npm install -g codexline

## Environment variables

- CODEXLINE_SKIP_DOWNLOAD=1: skip postinstall download.
- CODEXLINE_VERSION: override download version.
- CODEXLINE_BASE_URL: override release download URL base (default ends with /v).
- CODEXLINE_DOWNLOAD_RETRIES: max download retry count (default: 3).
- CODEXLINE_DOWNLOAD_TIMEOUT_MS: HTTP timeout in ms (default: 20000).
- CODEXLINE_VERIFY_CHECKSUM=0: disable checksum verification.
- CODEXLINE_REQUIRE_CHECKSUM=1: fail install if checksum file/entry is missing.
