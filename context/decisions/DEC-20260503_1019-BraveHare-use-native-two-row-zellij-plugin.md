---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260503_1019-BraveHare-use-native-two-row-zellij-plugin
  created: '2026-05-03T10:19:36+00:00'
spec:
  title: Use native two-row Zellij plugin surface for aibox key hints and runtime
    status
  state: accepted
  decision: 'Implement the long-term aibox Zellij bottom UI as a native two-row plugin
    surface: one 1-row key-hint bar and one 1-row runtime status bar. Each row must
    be independently showable/hidable from aibox leader-key bindings, and hiding a
    row should reclaim terminal space rather than rendering a blank line.'
  context: The current aibox Zellij layout uses the built-in zellij:status-bar plus
    a shell-pane aibox-status helper. The built-in keybar is sparse because aibox
    clears default keybindings and maps most commands behind Ctrl+g via Zellij tmux
    mode. The owner approved a refined plan for a first-class plugin surface after
    discussing the limitations of the built-in status/keybar.
  rationale: A native plugin can consume Zellij mode/keybinding state and render grouped
    hints that match aibox workflows, while a shell-pane status line cannot integrate
    cleanly with the keybar or avoid layout churn. Two separate rows keep key-hint
    and runtime concerns visually distinct and allow users to hide either surface
    when space or concentration matters.
  alternatives:
  - option: Keep built-in zellij:status-bar plus shell aibox-status
    assessment: Short-term fallback only. It preserves current behavior but cannot
      show the full grouped aibox leader map and depends on layout-pane redraw mechanics.
  - option: Use a single combined key/status row
    assessment: Rejected as default target because the combined content is too dense
      and will degrade quickly on narrower terminals.
  - option: Reconfigure aibox to use only Zellij native modes
    assessment: Useful partial improvement, but it still does not provide the grouped
      aibox-specific two-row UX or independent show/hide controls.
  consequences: The backlog item should target a Rust/WASM Zellij plugin or plugin
    pair, seeded into generated layouts, with the shell status helper retained as
    fallback until plugin behavior is verified. Implementation must include width-aware
    grouping, no refresh blink, independent leader-key toggles, and documentation
    of the migration path.
  deciders:
  - TEAMMEMBER-thrifty-otter
  related_workitems:
  - BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  decided_at: '2026-05-03T10:19:36+00:00'
---
