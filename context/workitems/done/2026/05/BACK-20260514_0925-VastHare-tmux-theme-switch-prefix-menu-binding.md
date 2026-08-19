---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260514_0925-VastHare-tmux-theme-switch-prefix-menu-binding
  created: '2026-05-14T09:25:15+00:00'
  updated: '2026-08-19T03:38:09+00:00'
spec:
  title: 'Tmux theme switch: prefix-key menu with two-tier live refresh'
  state: done
  type: story
  priority: medium
  description: |
    ## Goal

    Let users switch the aibox theme without exiting the container or losing their tmux session. Two tiers:

    - **Tier 1 (default `Prefix + T` → theme entry)** — light swap:
      1. `aibox theme <name>` regenerates managed files.
      2. `tmux source-file ~/.config/tmux/tmux.conf` re-loads tmux + PowerKit chevrons. Starship, git+delta, lazygit-next-launch already auto-pick up new colors.
      3. For each pane whose `pane_current_command` is bash/zsh/fish, send `\\e\\C-e\\C-u . ~/.config/aibox/theme-env.sh\\n`.
      4. For each `vim`/`nvim` pane: send `Escape :colorscheme aibox <CR>`.
      5. For each `yazi` pane: send `:plugin theme-reload` (Yazi 25.5+) or display fallback message.

    - **Tier 2 (default `Prefix + T` → "Heavy: restart TUIs")** — kill+respawn lazygit / lnav / Claude / Codex / Aider / Gemini / OpenCode panes (factored from layout-switch helper).

    ## Architecture

    1. **`~/.local/bin/aibox-tmux-refresh-theme [--restart-tuis]`** helper (~120 LOC bash):
       - Steps 2–5 above.
       - `--restart-tuis` flag invokes step 6: kill+respawn known TUI panes.
       - Uses `pane_current_command` guard to avoid sending keystrokes to non-shell panes that happen to look like bash.
       - Vim hot-reload via `Escape : colorscheme aibox CR` to survive INSERT mode.

    2. **`cli/src/tmux/status.rs`**: emit menu binding when `enabled = true`. Menu items populated dynamically from `customization.tmux.theme_switch.themes` plus a `Toggle light/dark` entry.

    3. **`aibox.toml` schema**:
       ```toml
       [customization.tmux.theme_switch]
       enabled    = true
       prefix_key = "T"
       themes     = ["gruvbox-dark", "catppuccin-mocha", "tokyo-night", "dracula", "projectious"]
       include_mode_toggle = true
       confirm_restart_tuis = true  # heavy tier shows kill-pane dialog
       ```

    4. **`cli/src/seed.rs`** writes the helper as a managed executable.

    ## Confirmation dialog

    - **Tier 1**: no confirmation. Light swap is non-destructive — open TUIs see old colors until reopened; Starship/tmux/Vim/Yazi/bash live-reload.
    - **Tier 2**: confirmation default ON, listing impacted panes from `pane_current_command`. Skippable via `confirm_restart_tuis = false`.

    ## Risks (per review)

    - Send-keys into non-shell panes mistyped as bash → guarded by `pane_current_command`. Worst case echoed.
    - Vim INSERT mode → `Escape` prefix.
    - Yazi < 25.5 → graceful fallback message.
    - Remote SSH panes look like bash but send-keys lands on the remote shell → false positive class, documented.

    ## Tests

    - Unit test: tmux.conf contains theme-switch binding when enabled; menu themes match config list.
    - Tier 3 vt100 test in `visual_rendered_tmux.rs`: pre-state catppuccin-mocha; drive `prefix + T` → pick dracula; capture: status bar surface bg should equal dracula surface.

    ## Delivery

    CLI-side only. No image rebuild required.
  started_at: '2026-05-14T09:46:39+00:00'
  completed_at: '2026-08-19T03:38:09+00:00'
---

## Transition note (2026-05-14T09:46:39+00:00)

Foundation shipped: schema (TmuxThemeSwitchSection with themes list + include_mode_toggle + confirm_restart_tuis), tmux.conf binding emission (display-menu populating themes dynamically, "Toggle light/dark" entry, "Heavy: restart TUIs" entry gated by AIBOX_THEME_CONFIRM_RESTART_TUIS env), aibox-tmux-refresh-theme helper script wired through seed.rs (send-keys hot-reload for bash/zsh/fish/vim/nvim/yazi; --restart-tuis flag for lazygit/lnav/AI panes). 947 unit tests green incl. 3 new theme_switch_* assertions; tier 1+3 e2e green; helper script passes bash -n.


## Transition note (2026-08-19T03:38:04+00:00)

Reconciled against current source: config, tmux bindings, managed helpers, rebuild/refresh behavior, confirmation flow, and executable seeding are implemented. Focused tmux status/layout/seed tests passed on 2026-08-19.


## Transition note (2026-08-19T03:38:09+00:00)

Review complete: implementation and focused regression tests satisfy the WorkItem scope. Archived as done.
