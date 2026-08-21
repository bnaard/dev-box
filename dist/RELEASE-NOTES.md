# aibox v0.34.2 — 2026-08-21

**Summary:** This patch adds a first-class theme gallery and chooser generated from the canonical theme correction data, aligns managed terminal colors with the design reference, and makes Codex/tmux attention state reliable after inline questions are answered.

## Added

- A Hugo theme overview with 76 generated variants, interactive qualitative filtering, per-theme configuration examples, and a dedicated tool-support reference.
- A managed Codex `notify` adapter that updates aibox attention state while preserving an existing project-local notify command.
- Explicit Yazi selection indicators and stronger active-pane styling for tmux panes containing Yazi.

## Changed

- Corrected generated theme foregrounds, active inks, syntax roles, and Contrast-family terminal previews against the Variant Board reference.
- Generate the documentation catalog from the same correction source used to scaffold addon theme files.
- Register v0.34.2 against processkit v0.28.8 in the CLI and documentation compatibility tables.

## Fixed

- Replace a stale Codex question marker with working state as soon as the inline answer reaches the transcript, and with done state when Codex reports turn completion.
- Avoid Vim `E418: Illegal value: dim` by omitting the unsupported highlight attribute.
- Make the active tmux pane visually distinct without depending on Yazi's internal pane selection.

## Upgrade notes

Upgrade the host CLI to v0.34.2 and run `aibox apply` so managed Codex hooks, tmux configuration, Vim colors, Yazi configuration, and theme files are regenerated. Existing Codex notify commands are retained behind the aibox adapter.

[v0.34.2]: https://github.com/projectious-work/aibox/compare/v0.34.1...v0.34.2
