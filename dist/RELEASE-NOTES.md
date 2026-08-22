# aibox v0.34.6 — 2026-08-22

**Summary:** This patch restores macOS host runtime image builds for the PowerKit separator fix. Use v0.34.6 for host Phase 2; v0.34.5 host builds cannot succeed because their immutable source contains the obsolete exact-count assertion.

## Added

- Add a regression that expands the PowerKit renderer with another legitimate conditional branch and verifies the compatibility patch remains valid.

## Changed

- Validate conditional comma escaping, canonical helper signatures and calls, and inherited-style resets structurally instead of counting global occurrences.

## Fixed

- Accept the nine escaped conditional color attributes in pinned PowerKit commit `6ac71f0d` so the runtime Docker target builds successfully.

## Removed

- Remove the brittle requirement that PowerKit contain exactly eight escaped conditional color attributes.

## Upgrade notes

Discard any prepared v0.34.5 host-run directory. Upgrade the host checkout to v0.34.6 and create a new host run; no processkit migration is required.

[v0.34.6]: https://github.com/projectious-work/aibox/compare/v0.34.5...v0.34.6
