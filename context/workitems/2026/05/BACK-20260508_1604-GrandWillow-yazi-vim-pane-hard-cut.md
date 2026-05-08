---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1604-GrandWillow-yazi-vim-pane-hard-cut
  created: '2026-05-08T16:04:11+00:00'
  labels:
    track: yazi-vim-hardcut
    release: v0.25.6
  updated: '2026-05-08T16:04:52+00:00'
spec:
  title: 'v0.25.6: Yazi/vim hard-cut — remove persistent vim pane, on-demand full-screen
    overlay'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal
    End the >20-release saga of the persistent-vim-pane fragility (yazi `e` repeatedly fails to load the file into the loop-running vim pane) by removing the persistent vim pane entirely. Owner-approved hard cut.

    ## Owner-specified behavior
    - **`e` on a marked file in yazi**: open a NEW full-screen pane on the current screen running vim with the file. `:q` in vim closes that pane and returns focus to yazi. No discovery of an existing vim pane.
    - **`Enter` on a file in yazi**: unchanged — vim opens in the yazi pane (yazi launcher pattern), `:q` closes vim and yazi resumes
    - **All layouts (`dev`, `ai`, `focus`, `cowork`, `cowork-swap`, `browse`)**: the persistent vim/editor pane is removed. Each layout becomes simpler.

    ## Scope

    ### 1. Layout regeneration
    - `cli/src/seed.rs:1400-1502` (`tmux_layout_script` + per-layout helpers, currently scheduled to move to `cli/src/tmux/layouts.rs` via BR-CODE-QUALITY Q3) — drop the editor pane creation from every layout
    - Update `images/base-debian/config/bin/aibox-tmux-session.sh` IMAGE variant to match
    - Layout templates that start with split-window for vim must be regenerated; the yazi pane becomes the sole content pane (or paired only with shell/AI panes per layout)

    ### 2. New `e` handler — full-screen popup
    - Replace `images/base-debian/config/bin/open-in-editor.sh` (current 128-line `find_editor_pane` machinery) with a ~15-line script that does:
      ```bash
      tmux display-popup -E -w 100% -h 100% -d "$(dirname "$file")" "${EDITOR:-vim} '$escaped_file'"
      ```
      `display-popup -E` is a blocking overlay that auto-closes when vim exits; covers the current pane edge-to-edge.
    - Drop `AIBOX_EDITOR_DIR`, `find_editor_pane`, `find_directional_pane`, `pane_is_editor`, `vim_return_cmd` — all gone.
    - Yazi keymap: keep `e` binding pointing at the new (smaller) script.

    ### 3. Yazi `Enter` opener — keep the suspending behavior
    - `.aibox-home/.config/yazi/yazi.toml` `[opener.edit]`: ensure `block = true` so yazi suspends until vim exits (current is `block = false`)
    - Verify `:q` returns to yazi cleanly

    ### 4. Tests + doctor
    - e2e: run yazi, simulate `e` keypress on a sample file, assert popup opens with vim and closes cleanly on `:q` returning focus to yazi
    - e2e: simulate `Enter` keypress, assert yazi suspends, vim opens, `:q` resumes yazi
    - doctor: warn if any layout still references the dropped editor pane (anti-regression check)
    - Remove tests that exercised the old `open-in-editor.sh` discovery logic

    ### 5. Docs
    - `AGENTS.md`: short note on the new `e` vs `Enter` semantics
    - `cheatsheet.txt`: update if it lists the old vim-pane behavior

    ## Why a hard cut
    Per owner: "I am seeing this misbehaviour now since more than 20 releases, appearing, fixed, re-appearing, that it is simply not possible to get to a stable implementation here." The complexity of reliably re-using a long-lived vim instance across tmux pane discovery, command sends, and pane-id tracking has not stabilized. Replacing with a stateless popup eliminates the entire failure surface.

    ## Acceptance
    - Zero `find_editor_pane` references in the codebase after this lands
    - All layouts have one less pane
    - `e` reliably opens a full-screen vim every single time
    - `Enter` retains its previous suspending-yazi behavior
    - All existing e2e tests pass after layout regen + new tests added
  blocked_by:
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
---
