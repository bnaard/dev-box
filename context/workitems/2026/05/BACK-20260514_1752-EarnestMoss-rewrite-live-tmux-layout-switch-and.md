---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260514_1752-EarnestMoss-rewrite-live-tmux-layout-switch-and
  created: '2026-05-14T17:52:59+00:00'
spec:
  title: Rewrite live tmux layout-switch and theme-switch e2e tests with asciinema
    capture
  state: backlog
  type: story
  priority: medium
  description: |
    v0.26.2 removed the broken Tier 3 visual_rendered_tmux.rs (6 tests) + visual_rendered_yazi.rs (1 test). Four of those tests were redundant with visual.rs (asciinema). The remaining two — `rendered_tmux_layout_switch_rebuilds_windows_to_focus` and `rendered_tmux_theme_switch_changes_status_bar_surface` — cover live interactions (Prefix+L layout chooser; Prefix+T theme chooser) that no other test covers, but used the broken `tmux capture-pane` approach which never sees the status bar.

    ## Rewrite plan

    Adopt the visual.rs `record_tmux` / `tmux_driver` pattern:

    1. Use `asciinema rec --cols 160 --rows 45 --overwrite -c driver.sh recording.cast` so the cast captures the full client view including the status bar.
    2. In the driver, invoke the actual switcher helpers (`aibox-tmux-switch-layout focus`, `aibox-tmux-refresh-theme`) against the live attached client.
    3. Parse `recording.cast` for: (a) post-switch window list (`tmux list-windows`), (b) status-bar surface color cells after theme switch.

    ## Acceptance

    - `cargo test --features "e2e e2e-render" --test e2e rendered_tmux_layout_switch_rebuilds_windows_to_focus` passes — verifies live layout switch rebuilds windows to the focus layout.
    - `cargo test --features "e2e e2e-render" --test e2e rendered_tmux_theme_switch_changes_status_bar_surface` passes — verifies live theme switch repaints the status bar in the new theme's surface color.
    - The new tests live in a fresh file (e.g. `visual_rendered_live_switches.rs`) wired via `cmd_test_e2e_render_layout_switch` and `cmd_test_e2e_render_theme_switch` in scripts/maintain.sh.
    - The `aibox-tmux-switch-layout.sh` socket-threading fix from v0.26.2 stays in place — the live-switch test should exercise that code path under a non-default `AIBOX_TMUX_SOCKET`.

    ## Context

    - Removed in commit (v0.26.2 hotfix); old visual_rendered_tmux.rs used capture-pane, structurally unable to assert tmux status-bar content.
    - `cli/tests/e2e/visual.rs::record_tmux` and `::tmux_driver` are the reference pattern.
    - `cli/src/templates/aibox-tmux-switch-layout.sh` + `aibox-tmux-confirm-and-switch.sh` + `aibox-tmux-refresh-theme.sh` now thread `-S "$socket"` via a `tmux()` shell function.
---
