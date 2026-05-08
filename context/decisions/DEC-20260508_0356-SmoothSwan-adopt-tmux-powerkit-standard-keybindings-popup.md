---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_0356-SmoothSwan-adopt-tmux-powerkit-standard-keybindings-popup
  created: '2026-05-08T03:56:20+00:00'
  updated: '2026-05-08T03:56:29+00:00'
spec:
  title: Adopt tmux-powerkit standard keybindings popup for in-session key help
  state: accepted
  decision: Use tmux-powerkit's built-in keybindings viewer popup as the default key-help
    mechanism (on-demand via prefix key), instead of implementing a persistent zellij-style
    keybar.
  rationale: This follows tmux-powerkit's intended interaction model, avoids custom
    runtime/UI complexity, and reduces maintenance risk while still giving discoverable
    key-help.
  consequences: Key help is user-invoked (popup) rather than always visible. If persistent
    hints are still desired later, implement pane-border hints as additive behavior
    without replacing the popup standard.
  related_workitems:
  - BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  - BACK-20260508_0356-QuickGarnet-implement-standard-tmux-powerkit-keybindings-popup
  decided_at: '2026-05-08T03:56:20+00:00'
---
