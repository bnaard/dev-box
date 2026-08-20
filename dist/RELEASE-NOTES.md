# aibox v0.34.1 — 2026-08-20

**Summary:** This patch release delivers coherent, configurable color themes across the managed terminal toolchain and restores the public theme gallery with faithful screenshots. Codex users also receive exact syntax palettes and a lifecycle fix that clears the question marker after permission prompts are answered.

## Added

- Expanded semantic theme palettes covering terminal chrome, tmux, shell tools, editors, syntax highlighting, and supported coding-agent harnesses.
- Generated TextMate themes consumed by bat, delta, and Codex for exact per-project syntax colors.
- A screenshot-based theme gallery with all supported theme variants and a reproducible capture script.

## Changed

- Extended theme configuration and documentation to describe every colored semantic token and each tool's native or generated theme support.
- Replaced fragile terminal recordings in the documentation gallery with deterministic rendered screenshots.
- Refreshed the generated v0.34.0 dogfood runtime and archived completed migration artifacts.

## Fixed

- Clear Codex's `question` lifecycle state after a permission response so the tmux title resumes its working indicator.
- Preserve palette-specific selection foreground/background values and light/dark terminal chrome across generated tool configs.
- Generate Codex theme files only when the Codex harness is enabled.

## Upgrade notes

Upgrade the host CLI to v0.34.1 and run `aibox apply` to regenerate managed terminal and harness theme files. Existing theme names remain compatible.

[v0.34.1]: https://github.com/projectious-work/aibox/compare/v0.34.0...v0.34.1
