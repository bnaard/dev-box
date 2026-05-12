---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0726-HappyFjord-implement-dec-trueclover-per-layout-multi
  created: '2026-05-10T07:26:27+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    depends-decision: DEC-TrueClover
  updated: '2026-05-10T07:48:45+00:00'
spec:
  title: 'Implement DEC-TrueClover: per-layout multi-harness behaviour for browse / cowork / cowork-swap / dev / focus'
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:48:17+00:00'
  completed_at: '2026-05-10T07:48:45+00:00'
---

## Transition note (2026-05-10T07:48:45+00:00)

Implemented and merged in commit 42a5b5c + merge 1c3c885. Per-layout multi-harness behaviour for browse (hidden), cowork/cowork-swap (stacked hidden panes via select-pane -d, prefix j/k cycle), dev/focus (secondary harnesses as named windows). New helpers cowork_secondary_panes / dev_secondary_windows / focus_secondary_windows in cli/src/tmux/layouts.rs. 15 new tests pass. DEC-20260510_0346-TrueClover satisfied.
