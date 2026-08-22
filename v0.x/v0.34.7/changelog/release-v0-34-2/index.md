# v0.34.2 — accurate themes and reliable agent state

> Adds the theme gallery and chooser, aligns generated themes with their references, and fixes Codex and tmux attention state.


The v0.34.2 release adds a first-class theme overview, interactive chooser, and
tool-support reference generated from the same correction data used to scaffold
managed theme files. Terminal previews now reflect the actual tmux, editor, and
tool colors, including corrected Contrast-family foregrounds and active-pane
indicators.

This patch also makes Codex completion notifications reliably replace a stale
question marker with working or done state, preserves project-local Codex
notification commands, strengthens the active tmux pane containing Yazi, and
avoids Vim errors from unsupported `dim` attributes.

[Full v0.34.2 release notes](https://github.com/projectious-work/aibox/releases/tag/v0.34.2)


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.7/changelog/release-v0-34-2/index.md
