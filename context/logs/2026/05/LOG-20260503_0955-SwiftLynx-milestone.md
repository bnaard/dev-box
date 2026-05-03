---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260503_0955-SwiftLynx-milestone
  created: '2026-05-03T09:55:04+00:00'
spec:
  event_type: milestone
  timestamp: '2026-05-03T09:55:04+00:00'
  summary: Expanded native Zellij status plugin scope to a two-line key-hint and runtime-status
    plugin target.
  actor: Codex
  subject: BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  subject_kind: WorkItem
  details:
    revised_title: Build native Zellij plugin for aibox key hints and runtime status
    target: 'Two-line bottom UI implemented as a native Zellij plugin surface: one
      key-hint bar and one runtime status bar, each independently showable/hidable.'
    findings:
    - The current aibox Zellij config clears default keybindings and leaves Normal
      mode with only Ctrl+g, so the built-in status bar only has a sparse Normal-mode
      map plus shared Alt bindings to show.
    - The aibox leader map currently lives in Zellij tmux mode, which works mechanically
      but prevents the built-in bar from presenting a curated all-leader-command menu
      while still in Normal mode.
    - Zellij's plugin API exposes ModeInfo.keybinds grouped by input mode, so a native
      plugin can render a purpose-built keybar from the real configured bindings rather
      than hardcoded documentation.
    - The built-in status-bar and compact-bar aliases define row slots but not a configurable
      grouped key-hint/status composition, so aibox needs its own plugin for the desired
      combined UX.
    scope:
    - Replace the shell-pane aibox-status workaround with a native Zellij plugin or
      plugin pair.
    - 'Render a 1-row key-hint bar grouped by task area: pane, tab, view/layout, tools,
      scroll/search, session.'
    - 'Render a 1-row runtime status bar grouped by system/project area: memory pressure,
      CPU throttling, process/processkit mode, filesystem, uptime, and project state.'
    - 'Support independent leader-key toggles for key hints and runtime status: show/hide
      keybar without hiding status, and show/hide status without hiding keybar.'
    - Adapt to terminal width by dropping lower-priority groups before truncating
      important labels.
    - Avoid layout churn and visual blinking on refresh.
    - Use live Zellij mode/keybinding data where possible, with a stable aibox fallback
      grouping for leader commands.
    acceptance_criteria:
    - A user can toggle the key-hint bar and status bar independently from the aibox
      leader key.
    - The key-hint row shows basic Zellij/aibox functions missing from the current
      sparse built-in bar, including open pane, close pane, split pane, fullscreen,
      pane navigation, tab navigation, scroll/search, and status toggles.
    - The status row keeps the accepted grouped runtime stats and does not blink during
      refresh.
    - The implementation does not require parsing or hand-maintaining generated keybinding
      comments as the primary data source if Zellij ModeInfo is available.
    - The design documents the tradeoff between replacing the built-in zellij:status-bar
      and running alongside it during migration.
---
