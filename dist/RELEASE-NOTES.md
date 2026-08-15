# aibox v0.32.5 — 2026-08-15

**Summary:** This patch makes addon discovery reliable for derived projects and improves text-copy workflows in Yazi and Vim. Users upgrading from an older or partially refreshed catalog no longer need to repair addon YAML files manually.

## Added

- Added `c c` in Yazi to copy the hovered file's contents to the tmux/host clipboard.
- Added `w v` in Yazi to open a selectable, read-only Vim view of the hovered file.

## Changed

- The CLI now embeds the complete canonical addon catalog and treats installed catalog files as optional name-based overrides.
- Reinstalling the same aibox version refreshes both the executable and addon catalog instead of exiting early.
- The Yazi cheatsheet documents horizontal preview scrolling and selectable preview alternatives.

## Fixed

- Fixed valid configured addons such as `supply-chain`, `browser-testing`, `go-quality`, and `release` being skipped when a host had a stale or incomplete addon catalog.
- Fixed Vim visual yanks so the selected text is forwarded to the tmux/host clipboard.

## Upgrade notes

Upgrade the host CLI to v0.32.5 and run `aibox apply`. No manual addon-catalog repair is required.

[v0.32.5]: https://github.com/projectious-work/aibox/compare/v0.32.4...v0.32.5
