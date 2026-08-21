# aibox v0.34.3 — 2026-08-21

**Summary:** This patch makes themed terminal state reliably readable and restores terminal ownership when leaving aibox. Projects using Yazi, tmux, or Codex should upgrade and run `aibox apply` to regenerate the corrected runtime configuration.

## Changed

- Use Codex's supported project-local `Stop` lifecycle hook for completion state instead of the unsupported project-local `notify` configuration key.
- Save the outer terminal title before attaching tmux and restore it when the aibox session exits.
- Keep inactive and active tmux pane surfaces authoritative after PowerKit renders its status layout.
- Register v0.34.3 against processkit v0.28.8 in the CLI and documentation compatibility tables.

## Fixed

- Keep selected, copied, and cut Yazi entries readable across every generated theme by coloring marker glyphs without painting filenames with the same color.
- Give Yazi's current-column indicator contrasting foreground and background colors.
- Remove malformed trailing `#` characters from PowerKit window-separator color attributes that produced dark or mismatched divider wedges.
- Clear retained `set-titles-string` state when project title management is disabled, preventing stale agent titles after reloads and exits.
- Preserve and restore a user's pre-existing Codex notification command when migrating away from the aibox-managed legacy adapter.

## Upgrade notes

Upgrade the host CLI to v0.34.3 and run `aibox apply`. Recreate or rebuild the container to receive the corrected baked PowerKit renderer; restart existing Yazi processes so they reload the generated theme.

[v0.34.3]: https://github.com/projectious-work/aibox/compare/v0.34.2...v0.34.3
