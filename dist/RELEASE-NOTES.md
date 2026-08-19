# aibox v0.33.2 — 2026-08-19

**Summary:** This patch release updates the default processkit release, refreshes the generated development runtime for the latest resolved Codex package, and records the already-shipped tmux layout and theme switching workflows as complete. Existing v0.33 users can upgrade normally; no configuration migration is required.

## Changed

- Refreshed generated runtime metadata to resolve Codex 0.148.0.
- Updated the default processkit release from v0.28.6 to v0.28.8 with unchanged format vocabulary.
- Reconciled the live tmux layout chooser and two-tier theme refresh implementation with their processkit WorkItems.

## Fixed

- Restored the generic lifecycle fallback in installed layout snapshots for harnesses that do not expose native attention hooks.

## Upgrade notes

Upgrade the host CLI to v0.33.2 and run `aibox apply` to refresh managed runtime files.

[v0.33.2]: https://github.com/projectious-work/aibox/compare/v0.33.1...v0.33.2
