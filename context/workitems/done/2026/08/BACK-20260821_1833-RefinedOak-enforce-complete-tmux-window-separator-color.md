---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1833-RefinedOak-enforce-complete-tmux-window-separator-color
  created: '2026-08-21T18:33:58+00:00'
  updated: '2026-08-21T18:35:57+00:00'
spec:
  title: Enforce complete tmux window separator color sequence
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: 'Implement the owner-specified first-row sequence: rounded session
    ending; status-background incoming arrow; light active index and arrow; active
    name background and outgoing arrow; status-background incoming arrow; then corresponding
    inactive light index, arrow, name background, and outgoing arrow. Cover the complete
    ordered sequence in isolated real tmux and reconcile the live renderer.'
  parent: BACK-20260821_1805-NobleOak-correct-tmux-window-separator-color-continuity
  started_at: '2026-08-21T18:34:02+00:00'
  completed_at: '2026-08-21T18:35:57+00:00'
---

## Transition note (2026-08-21T18:34:02+00:00)

Owner supplied exact ordered color semantics; reverting the incorrect destination-name-colored incoming chevron and testing the full sequence.


## Transition note (2026-08-21T18:35:56+00:00)

Implemented and live-verified the owner-specified rounded session and full status-gap/index/name/outgoing color sequence; ordered real-tmux regression passes.


## Transition note (2026-08-21T18:35:57+00:00)

Verified live four-window format, rounded session glyph, exact active/inactive color chain, escaped conditional commas, patch idempotence, Bash syntax, ShellCheck, visual regression, and git diff check.
