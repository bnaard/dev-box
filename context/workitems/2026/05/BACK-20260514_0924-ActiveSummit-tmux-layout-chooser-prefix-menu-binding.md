---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260514_0924-ActiveSummit-tmux-layout-chooser-prefix-menu-binding
  created: '2026-05-14T09:24:56+00:00'
  updated: '2026-05-14T09:46:32+00:00'
spec:
  title: 'Tmux layout chooser: prefix-key menu for live layout switching'
  state: in-progress
  type: story
  priority: medium
  description: |
    ## Goal

    Let users switch tmux layouts (ai / dev / focus / cowork + user-defined) without exiting the container. Trigger via the configured aibox prefix key + a layout-chooser key, presenting a `display-menu`. Selecting an entry rebuilds windows in the *attached* session.

    ## Architecture (shape C from review — menu-driven, kill-windows-in-place)

    1. **`~/.local/bin/aibox-tmux-switch-layout <name>`** helper (~80 LOC bash):
       - Reads current session via `tmux display-message -p '#S'`.
       - Renames window 0 to `_swap_`, kills all other windows.
       - Executes `~/.config/tmux/layouts/<name>.sh` with `AIBOX_LAYOUT_MODE=rebuild` env set so the script skips its `has-session` short-circuit.
       - Kills the `_swap_` window after rebuild, selects window 0.

    2. **`cli/src/tmux/layouts.rs` — `tmux_layout_script` refactor**: add a `rebuild` branch that runs only the window-creation block, no `new-session`, no `attach-session`. Existing fresh-start path stays default.

    3. **`cli/src/tmux/status.rs` — tmux.conf binding emission** (menu-driven default).

    4. **`aibox.toml` schema**:
       ```toml
       [customization.tmux.layout_switch]
       enabled    = true
       prefix_key = "L"
       style      = "menu"     # "menu" | "table"
       confirm    = true       # show "this will kill open panes" dialog
       ```

    5. **`cli/src/seed.rs`** writes the helper as a managed executable.

    ## Confirmation dialog

    Layout switch is **always destructive** — running TUI processes in the session's panes die. Default = pre-execution dialog naming the impacted apps from `pane_current_command` inventory. Skippable via `confirm = false`.

    ## Tests

    - Unit test: tmux.conf contains binding when enabled.
    - Unit test: rebuild mode emits no new-session / attach-session.
    - Tier 3 vt100 test in `visual_rendered_tmux.rs`: drive `prefix + L + d`, capture, assert rendered window list matches dev layout.

    ## Delivery

    CLI-side only. `aibox apply` regenerates tmux.conf + the helper script. No image rebuild required.
  started_at: '2026-05-14T09:46:32+00:00'
---

## Transition note (2026-05-14T09:46:32+00:00)

Foundation shipped: schema (TmuxLayoutSwitchSection), rebuild-mode branch (_create_first_window function), tmux.conf binding emission (display-menu default with confirm dispatcher), three managed helper scripts wired through seed.rs. 947 unit tests green; tier 1+3 e2e green; helper scripts pass bash -n.
