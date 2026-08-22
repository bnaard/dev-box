# v0.34.3 — readable themes and restored terminal titles

> Repairs Yazi marks and PowerKit dividers across all themes, preserves active-pane emphasis, and restores terminal titles on exit.


The v0.34.3 patch makes marked Yazi entries readable across the complete theme
catalogue and repairs malformed PowerKit window-separator colors that could
produce dark or mismatched divider wedges. Active tmux panes retain their
distinct surface and border after PowerKit loads.

Codex completion state now uses the supported project-local `Stop` hook instead
of a rejected project-local `notify` key. Tmux also saves the outer terminal
title before attachment, clears retained title expressions when disabled, and
restores terminal ownership when the aibox session exits.

After upgrading, run `aibox apply`; rebuild the container for the baked
PowerKit correction and restart existing Yazi processes to reload their theme.

[Full v0.34.3 release notes](https://github.com/projectious-work/aibox/releases/tag/v0.34.3)


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.5/changelog/release-v0-34-3/index.md
