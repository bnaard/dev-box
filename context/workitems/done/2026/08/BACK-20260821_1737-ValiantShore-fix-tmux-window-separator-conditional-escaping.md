---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1737-ValiantShore-fix-tmux-window-separator-conditional-escaping
  created: '2026-08-21T17:37:53+00:00'
  updated: '2026-08-21T17:53:01+00:00'
spec:
  title: Fix tmux window separator conditional escaping
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Correct the v0.x PowerKit window-separator source patch that removes
    tmux conditional comma escaping, add isolated real-tmux regression coverage for
    rendered window spacing and absence of leaked color attributes, reconcile the
    local installed runtime, and verify the live status bar.
  started_at: '2026-08-21T17:38:02+00:00'
  completed_at: '2026-08-21T17:53:01+00:00'
---

## Transition note (2026-08-21T17:38:02+00:00)

Root cause reproduced in isolated tmux; implementing source patch correction, regression coverage, and local runtime reconciliation.


## Transition note (2026-08-21T17:53:01+00:00)

Source patch now restores tmux conditional comma escaping; isolated real-tmux regression passes; local installed renderer patched and live status reloaded cleanly.


## Transition note (2026-08-21T17:53:01+00:00)

Verified patch idempotence, Bash syntax, shellcheck, git diff check, real-tmux two-window spacing, absence of leaked bg attributes, and live four-window header rendering.
