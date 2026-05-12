---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260505_1202-JollyLily-use-custom-native-zellij-keybar-and
  created: '2026-05-05T12:02:02+00:00'
spec:
  title: Use custom native Zellij keybar and statusbar styling
  state: accepted
  decision: Implement aibox native Zellij UI as a custom key indication bar plus a visually consistent runtime status bar, rather than relying on Zellij's built-in status bar for native mode.
  context: 'The owner approved the custom keybar direction with amendments: key previews must include compact action descriptions, visuals should mimic Zellij''s arrow-like built-in style, keybar and runtime status bar should share one visual language, the status bar can use more width for additional stats, and the runtime status should include a Zellij mode indicator.'
  rationale: The built-in Zellij bar is visible and familiar but cannot show aibox-specific leader-key previews and custom runtime state. A custom native plugin can consume Zellij mode/keybinding events, mimic the built-in visual language, and add aibox-specific hints and status groups.
  alternatives:
  - option: Use Zellij built-in status bar in native mode
    status: rejected
    reason: It works visually but cannot provide aibox-specific leader previews and extended status groups.
  - option: Keep hardcoded terse aibox key hints
    status: rejected
    reason: It lacks descriptions for keys such as h/j/k/l, n, d, r, and x, making it less useful than the built-in bar.
  consequences: Native mode should mount the aibox plugin for both key and runtime rows. The implementation must keep width-aware rendering, include tests for mode-specific key hints and status layout, and preserve a fallback path for shell/built-in status mode.
  related_workitems:
  - BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  decided_at: '2026-05-05T12:02:02+00:00'
---
