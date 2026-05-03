---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260503_1019-SoundSpruce-milestone
  created: '2026-05-03T10:19:51+00:00'
spec:
  event_type: milestone
  timestamp: '2026-05-03T10:19:51+00:00'
  summary: Recorded approved implementation plan for native two-row Zellij key-hint
    and runtime-status plugin.
  actor: Codex
  subject: BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  subject_kind: WorkItem
  details:
    decision_record: DEC-20260503_1019-BraveHare-use-native-two-row-zellij-plugin
    approved_plan:
      target: 'Replace the shell-pane aibox-status workaround with a native Zellij
        bottom UI made of two independently showable/hidable 1-row surfaces: key-hint
        bar and runtime-status bar.'
      core_design:
      - Build one Rust/WASM Zellij plugin binary loaded twice with mode=keybar and
        mode=status, or split into two plugins if Zellij pane lifecycle behavior makes
        that cleaner.
      - Seed layouts with two size=1 borderless plugin panes instead of the current
        zellij:status-bar plus shell aibox-status pane once behavior is verified.
      - Keep both rows non-focusable so user focus never lands on status UI.
      - Use Zellij ModeUpdate/ModeInfo.keybinds as the primary source for keybinding
        state; keep an explicit aibox grouping map only where needed for workflow
        labeling.
      key_hint_row:
      - 'Group hints by workflow: PANE, TAB, VIEW, TOOLS, SCROLL, SESSION.'
      - Show compact leader-available hints in Normal mode.
      - Show full grouped leader map in the leader/Tmux mode.
      - Drop lower-priority groups by terminal width before truncating important labels.
      status_row:
      - Render MEM, CPU, PROC, FS, UP, and PROJ groups.
      - Use Timer refreshes for runtime data.
      - Refresh project-state checks more slowly than cgroup/proc checks.
      - Use direct lightweight reads where possible and a JSON helper command fallback
        if WASM access is constrained.
      toggle_model:
      - Provide independent leader toggles for keybar and status row.
      - Use Ctrl+g k for key-hint row and Ctrl+g v for status row unless implementation
        reveals a conflict.
      - Prefer true hide via plugin pane hide/show; if that does not reflow reliably,
        use generated layout variants for both/key-only/status-only/neither.
      - Optionally add Ctrl+g b later to cycle both/key/status/none.
      migration_path:
      - Keep current shell aibox-status as fallback.
      - Add plugin build, seed, and install pipeline.
      - Add new plugin-row layouts and toggle bindings.
      - Update docs and quick reference.
      - Remove shell-pane status from default layouts only after plugin behavior is
        verified.
      acceptance_criteria:
      - Keybar and status row can be toggled independently.
      - Hiding either row reclaims terminal space.
      - No refresh blink.
      - Keybar shows missing basic Zellij/aibox functions such as open pane, close
        pane, split pane, fullscreen, pane navigation, tab navigation, scroll/search,
        and status toggles.
      - Status row preserves the accepted grouped runtime stats.
      - Generated layouts pass zellij setup --check.
      - Shell status remains as fallback until plugin stability is proven.
      risks:
      - Confirm hidden plugin panes reflow layout cleanly.
      - Confirm filesystem/proc access from WASM in this container context or rely
        on run_command fallback.
      - Confirm how much of leader grouping can be inferred from ModeInfo.keybinds
        versus explicit grouping metadata.
---
