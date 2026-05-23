---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260521_1024-ToughSky-aibox-v0-27-2-release-complete
  created: '2026-05-21T10:24:56+00:00'
spec:
  title: 'Session handover: aibox v0.27.2 release complete'
  body: |
    ## Summary

    Completed the aibox v0.27.2 patch release for the Hugo checksum verification fix.

    ## Completed

    - Fixed `docs-hugo` runtime checksum verification so Hugo archives downloaded to `/tmp/hugo.tar.gz` are checked against the matching Hugo release checksum entry after rewriting the checksum path.
    - Added regression coverage in `cli/src/addon_loader.rs`.
    - Added `0.27.2` to `cli/src/compat.rs` and `docs-site/docs/reference/compatibility.md`.
    - Released with expensive companion E2E/visual tiers skipped via `--skip e2e,visual`; normal fmt, clippy, unit tests, tier-1/no-container E2E, local Starship render tests, cargo audit, Linux builds, and version smoke passed.
    - Pushed `main` and tag `v0.27.2`.
    - Created GitHub release `https://github.com/projectious-work/aibox/releases/tag/v0.27.2` with Linux binaries, macOS binaries, checksum sidecars, and LICENSE.
    - Deployed documentation to GitHub Pages.
    - Host-side `./scripts/maintain.sh release-host 0.27.2` completed: macOS binaries uploaded, container images pushed to GHCR and verified live, runtime smoke passed, generated runtime refreshed and committed if needed.

    ## Current Repo State

    - `main` HEAD: `fa8f7719 chore: refresh generated runtime for v0.27.2`.
    - Tag `v0.27.2` points to `1ed1d503 docs: add v0.27.2 compatibility entry`.
    - Release smoke artifacts are under `dist/release-smoke/v0.27.2/20260520-210149/`.
    - Working tree is not fully clean: `aibox.toml` has comment-catalog drift showing updated addon version option comments. This drift existed before the release and was intentionally kept out of the release commits.
    - `stash@{0}` is still `On main: keep-aibox-toml-comment-drift`, which appears to preserve the same `aibox.toml` comment drift now also visible in the working tree.

    ## Follow-up

    - Decide whether to commit, discard, or re-standardize the `aibox.toml` comment-catalog drift.
    - After that decision, remove the duplicate `stash@{0}` if it is no longer needed.
    - Derived projects affected by the Hugo build failure should update to aibox v0.27.2 and regenerate/rebuild their devcontainer.
  type: reference
  state: captured
  review_due: '2026-05-28'
  tags:
  - handover
  - release
  - aibox
  - v0.27.2
  source: pk-wrapup
---

## Summary

Completed the aibox v0.27.2 patch release for the Hugo checksum verification fix.

## Completed

- Fixed `docs-hugo` runtime checksum verification so Hugo archives downloaded to `/tmp/hugo.tar.gz` are checked against the matching Hugo release checksum entry after rewriting the checksum path.
- Added regression coverage in `cli/src/addon_loader.rs`.
- Added `0.27.2` to `cli/src/compat.rs` and `docs-site/docs/reference/compatibility.md`.
- Released with expensive companion E2E/visual tiers skipped via `--skip e2e,visual`; normal fmt, clippy, unit tests, tier-1/no-container E2E, local Starship render tests, cargo audit, Linux builds, and version smoke passed.
- Pushed `main` and tag `v0.27.2`.
- Created GitHub release `https://github.com/projectious-work/aibox/releases/tag/v0.27.2` with Linux binaries, macOS binaries, checksum sidecars, and LICENSE.
- Deployed documentation to GitHub Pages.
- Host-side `./scripts/maintain.sh release-host 0.27.2` completed: macOS binaries uploaded, container images pushed to GHCR and verified live, runtime smoke passed, generated runtime refreshed and committed if needed.

## Current Repo State

- `main` HEAD: `fa8f7719 chore: refresh generated runtime for v0.27.2`.
- Tag `v0.27.2` points to `1ed1d503 docs: add v0.27.2 compatibility entry`.
- Release smoke artifacts are under `dist/release-smoke/v0.27.2/20260520-210149/`.
- Working tree is not fully clean: `aibox.toml` has comment-catalog drift showing updated addon version option comments. This drift existed before the release and was intentionally kept out of the release commits.
- `stash@{0}` is still `On main: keep-aibox-toml-comment-drift`, which appears to preserve the same `aibox.toml` comment drift now also visible in the working tree.

## Follow-up

- Decide whether to commit, discard, or re-standardize the `aibox.toml` comment-catalog drift.
- After that decision, remove the duplicate `stash@{0}` if it is no longer needed.
- Derived projects affected by the Hugo build failure should update to aibox v0.27.2 and regenerate/rebuild their devcontainer.
