# aibox v0.33.0 — 2026-08-17

**Summary:** This minor release makes AI-agent attention visible in tmux-owned terminal titles without binding the experience to a particular terminal or provider. Existing projects can opt into desktop-style notifications and customize title content, symbols, and limits through `aibox.toml`.

## Added

- Added configurable tmux terminal titles with project, window, directory, repository, branch, harness, agent, task, message, elapsed-time, and attention-state placeholders.
- Added provider-neutral `aibox-agent-signal` lifecycle integration for generated AI panes and supported harness hooks, including working, question, done, error, and idle states.
- Added optional OSC 9 or bell notifications with configurable states and message inclusion.

## Changed

- Generated tmux layouts now wrap AI harness processes with safe lifecycle fallbacks while preserving an interactive shell after completion or failure.
- Refreshed the generated v0.32.6 runtime and processkit installation baseline used by this project.

## Fixed

- Updated generated Yazi directory previews to use the Yazi 26 theme icon matcher instead of the removed file icon method.
- Serialized processkit gateway startup so concurrent harnesses do not race to launch the shared daemon.

## Upgrade notes

Upgrade the host CLI to v0.33.0 and run `aibox apply` to generate the attention helper, harness hooks, and tmux title configuration. Notifications remain disabled unless explicitly enabled.

[v0.33.0]: https://github.com/projectious-work/aibox/compare/v0.32.6...v0.33.0
