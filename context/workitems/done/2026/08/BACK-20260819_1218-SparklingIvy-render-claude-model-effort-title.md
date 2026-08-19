---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1218-SparklingIvy-render-claude-model-effort-title
  created: '2026-08-19T12:18:34+00:00'
  updated: '2026-08-19T12:22:42+00:00'
spec:
  title: Render Claude Code model and effort in tmux title
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Capture Claude Code model and effective effort from hook input/environment,
    persist runtime identity per pane, and prevent later lifecycle signals from clearing
    it.
  started_at: '2026-08-19T12:18:39+00:00'
  completed_at: '2026-08-19T12:22:42+00:00'
---

## Transition note (2026-08-19T12:18:39+00:00)

Official hook schema and live Claude pane inspected; implementation started.


## Transition note (2026-08-19T12:22:41+00:00)

Claude SessionStart and lifecycle hooks now pass hook input; helper resolves direct model/effort or latest transcript values and persists per-pane metadata. Focused real-tmux, hook registration, clippy, format, live apply, live Claude pane, and pk-doctor verified.


## Transition note (2026-08-19T12:22:42+00:00)

Live Claude pane renders aibox — claude-opus-5 medium@claude.
