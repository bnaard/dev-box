# aibox v0.33.1 — 2026-08-17

**Summary:** This patch release restores configured AI harness startup after applying v0.33.0 and keeps Yazi directory previews usable when the host CLI and configured image briefly differ. It also makes the tmux status bar easier to scan by matching PowerKit plugin transitions to the established window-tab spacing.

## Added

- Added the nearest reachable release tag to the Forge status segment.
- Added an isolated tmux/asciinema regression check for left- and right-side PowerKit plugin transitions.

## Changed

- PowerKit plugin spacing now uses side-aware two-chevron transitions matching the tmux window tabs, without rectangular spacer cells.
- The pinned PowerKit source patch is now fail-closed and idempotent so upstream drift cannot silently produce malformed status segments.

## Fixed

- Fixed generated AI harness layouts failing before Codex, Claude, or another configured harness could start because the lifecycle exit variable was expanded under nounset.
- Fixed Yazi directory previews crashing on the icon matcher when a v0.33 CLI refreshed managed files for a project still running an older configured image.
- Kept the Yazi compatibility fallback free of the deprecated file icon method.

## Removed

- Removed the legacy one-cell rectangular PowerKit spacing workaround.

## Upgrade notes

Upgrade the host CLI to v0.33.1, set release_version to latest or 0.33.1, then run aibox apply, aibox down, and aibox up to regenerate the layouts and align the runtime image.

[v0.33.1]: https://github.com/projectious-work/aibox/compare/v0.33.0...v0.33.1
