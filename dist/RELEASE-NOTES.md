# aibox v0.34.0 — 2026-08-19

**Summary:** This minor release makes tmux window titles agent-aware and moves the documentation website onto the projectious.work Hugo brand theme. Titles now identify the repository, active model, and harness, while lifecycle symbols expose working, question, completion, and error states without adding suffixes to ordinary shell or lazygit windows.

## Added

- Configurable tmux title formats, state symbols, agent suffixes, completion TTLs, and terminal-neutral attention notifications.
- Native Codex and Claude Code lifecycle integration for working, question, done, error, and idle transitions.
- A projectious.work-branded Hugo documentation site with a public Change log and two-phase Roadmap.

## Changed

- Replaced the legacy Hugo Docsy/Bootstrap/Font Awesome dependency stack with `brand-theme-hugo-vanilla` v0.3.4 and Hugo-native theme constructs.
- Updated the deployed documentation identity, navigation, compatibility matrix, and release archive.
- Temporarily removed the terminal theme gallery from publication until its recordings preserve each palette and Nerd Font symbols correctly.

## Fixed

- Refresh the complete tmux client after agent-state transitions so host terminal titles immediately repaint their lifecycle symbol.
- Omit the trailing `@` agent separator for non-agent windows.
- Keep tmux title ownership stable while switching between Codex, Claude Code, lazygit, and shell windows.

## Upgrade notes

Upgrade the host CLI to v0.34.0 and run `aibox apply` to regenerate the managed tmux configuration and lifecycle helper. Existing `customization.tmux.title` settings remain supported; projects without explicit title settings receive the new agent-aware defaults.

[v0.34.0]: https://github.com/projectious-work/aibox/compare/v0.33.2...v0.34.0
