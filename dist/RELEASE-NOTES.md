# aibox v0.34.5 — 2026-08-22

**Summary:** This patch gives every bundled theme the intended tmux window separators and makes visual regressions release-blocking. Upgrade the CLI and run `aibox apply` to refresh the managed tmux configuration.

## Added

- Add structural and rendered regressions for escaped tmux conditionals, separator color continuity, attribute resets, and all bundled themes.

## Changed

- Run visual E2E and the isolated all-theme cast sweep as mandatory release gates.
- Refresh deferred pins for Go, Rust, Bun, PDM, OpenTofu, kubectl, Tau, Zensical, and the Rust `cc` crate.

## Fixed

- Preserve PowerKit's escaped conditional commas so tmux no longer renders fragments such as `bg=#21252B]`.
- Give outgoing arrows the same background color as their window-name segment and reset inherited dim styling.
- Isolate every theme capture on its own tmux server and validate the palette actually loaded before recording.

## Removed

- Remove persistent documentation-cast writes from the release regression path; validation evidence is disposable and cannot overwrite the theme gallery.

## Upgrade notes

Upgrade the host CLI to v0.34.5 and run `aibox apply` to refresh the managed tmux configuration. No processkit migration is required.

[v0.34.5]: https://github.com/projectious-work/aibox/compare/v0.34.4...v0.34.5
