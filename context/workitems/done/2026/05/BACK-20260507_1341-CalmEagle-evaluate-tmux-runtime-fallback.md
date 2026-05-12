---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1341-CalmEagle-evaluate-tmux-runtime-fallback
  created: '2026-05-07T13:41:09+00:00'
  labels:
    area: terminal-multiplexer
    candidate: tmux
    risk: zellij-plugin-system
  updated: '2026-05-07T14:56:15+00:00'
spec:
  title: Evaluate tmux as a stable multiplexer alternative or fallback for aibox runtimes
  state: cancelled
  type: research
  priority: high
  description: 'Evaluate whether aibox should keep Zellij as default, add tmux as an alternative runtime multiplexer, or switch defaults. Context: repeated runtime failures point at Zellij plugin/server blast radius; Zellij Rust basis and server features are attractive, but plugin instability can pin the session server at 300%+ CPU. Compare tmux stability, status-line programmability, pane/tab/session model, keybinding ergonomics, macOS keyboard behavior, plugin/runtime observability, and migration cost for generated layouts. Acceptance: recommendation with concrete implementation path and release strategy.'
  scope: runtime-architecture
  completed_at: '2026-05-07T14:56:15+00:00'
---

## Transition note (2026-05-07T14:56:15+00:00)

Superseded by accepted decision DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime and implementation epic BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign.
