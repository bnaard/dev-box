# aibox v0.31.5 — 2026-08-13

**Summary:** This patch makes OpenCode-enabled container builds tolerate transient GitHub download failures and corrects Textual log-yank behavior in the macOS release gate. Operators can yank only the marked log range and receive useful diagnostic tails for failures that do not print an explicit error line; no configuration change is required.

## Added

- Add bounded failed-task output tails to the canonical Textual Problems bundle when command output lacks an explicit error classifier.

## Changed

- Bind lowercase `y` to the active Textual log selection, matching Vim visual-mode semantics; retain full selected-task copying on uppercase `Y`.

## Fixed

- Retry pinned OpenCode release archive and checksum downloads across transient connection failures, including curl exit 56.
- Retry both release discovery and archive downloads in the unpinned OpenCode fallback path.
- Preserve useful task output in Problems when a failed command exits non-zero without printing `error`, `fatal`, or a similar classifier.

[v0.31.5]: https://github.com/projectious-work/aibox/compare/v0.31.4...v0.31.5
