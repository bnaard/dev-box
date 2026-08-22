---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1805-NobleOak-correct-tmux-window-separator-color-continuity
  created: '2026-08-21T18:05:46+00:00'
  updated: '2026-08-21T18:24:14+00:00'
spec:
  title: Correct tmux window separator color continuity
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Correct remaining first-row PowerKit window separator colors so each
    arrow peak matches the adjacent window-name segment background. Update the source
    patch, isolated real-tmux color-continuity regression, local installed renderer,
    and live tmux state.
  parent: BACK-20260821_1737-ValiantShore-fix-tmux-window-separator-conditional-escaping
  started_at: '2026-08-21T18:05:50+00:00'
  completed_at: '2026-08-21T18:24:14+00:00'
---

## Transition note (2026-08-21T18:05:50+00:00)

Screenshot confirms syntax leak is fixed but separator foreground/background continuity remains incorrect; tracing exact rendered color pairs.


## Transition note (2026-08-21T18:24:13+00:00)

Incoming chevrons now inherit each destination window's content background; real-tmux regression validates active/inactive color equality and clean rendering.


## Transition note (2026-08-21T18:24:14+00:00)

Local renderer patched and live tmux reloaded; verified inactive fg #8B949E and active fg #58A6FF match their window-name backgrounds, plus syntax, ShellCheck, idempotence, visual test, and git diff check.
