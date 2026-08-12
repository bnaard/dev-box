# aibox v0.31.4 — 2026-08-12

**Summary:** This patch makes Hugo-enabled container builds tolerate transient GitHub download failures and makes release-host failures easier to select, inspect, and copy. Release and visual E2E gates now serialize contention-sensitive execution automatically; no configuration change is required.

## Added

- Add a canonical Textual Problems panel with warning/failure extraction, selection-aware copying, last-20-line selection, and error-only yanking.
- Add fixed-denominator numeric task progress and an accessible task-state legend to the release-host dashboard.

## Changed

- Run canonical test and local visual E2E release gates with one test thread to avoid shared-runtime contention.
- Keep generated processkit/runtime state aligned with processkit v0.28.6.

## Fixed

- Retry Hugo release archive and checksum downloads across transient connection failures, including curl exit 56.
- Exclude nested Git repositories and generated host-gate caches from application lockfile checks in `pk-doctor`.
- Preserve literal candidate text and authoritative plain-output evidence while improving the Textual presentation layer.

[v0.31.4]: https://github.com/projectious-work/aibox/compare/v0.31.3...v0.31.4
