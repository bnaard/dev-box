---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0336-SmartLark-tmux-layouts-multi-harness-ai-layout
  created: '2026-05-10T03:36:30+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    needs-decision: 'true'
  updated: '2026-05-10T07:14:33+00:00'
spec:
  title: 'tmux layouts: multi-harness ai-layout default geometry + leader keybindings + per-layout multi-harness proposals'
  state: done
  type: bug
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:14:07+00:00'
  completed_at: '2026-05-10T07:14:33+00:00'
---

## Transition note (2026-05-10T07:14:33+00:00)

Implemented and merged in commit 3d2d8d6 + merge ded4dd3. ai_secondary_panes() in cli/src/tmux/layouts.rs cascades split-window for harnesses 2..N (primary at ~80%); leader z explicit zoom binding added; leader j/k descriptions clarified to 'next/prev harness pane'. 5 new tests pass. Per-layout multi-harness defaults for browse/cowork/cowork-swap/dev/focus parked in DEC-20260510_0346-TrueClover (proposed); pending owner acceptance for follow-up implementation.
