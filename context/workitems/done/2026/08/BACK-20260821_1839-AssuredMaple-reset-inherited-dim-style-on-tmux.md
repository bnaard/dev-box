---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1839-AssuredMaple-reset-inherited-dim-style-on-tmux
  created: '2026-08-21T18:39:51+00:00'
  updated: '2026-08-21T18:57:40+00:00'
spec:
  title: Reset inherited dim style on tmux window arrows
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Ensure window separator glyphs reset inherited text attributes before
    applying foreground/background colors, so the outgoing arrow after inactive names
    such as ai renders the exact same gray rather than a dimmed variant. Add regression
    coverage and reconcile the live renderer.
  parent: BACK-20260821_1833-RefinedOak-enforce-complete-tmux-window-separator-color
  started_at: '2026-08-21T18:39:57+00:00'
  completed_at: '2026-08-21T18:57:40+00:00'
---

## Transition note (2026-08-21T18:39:57+00:00)

Confirmed outgoing color token is correct but inherits the inactive segment's dim attribute; resetting styles at separator boundaries.


## Transition note (2026-08-21T18:57:40+00:00)

Separator formats now reset inherited attributes before applying colors; live inactive outgoing arrow is #[none] with fg #8B949E matching the ai background.


## Transition note (2026-08-21T18:57:40+00:00)

Verified live reload, exact outgoing arrow style reset/color, complete real-tmux sequence regression, patch idempotence, Bash syntax, ShellCheck, and git diff check.
