---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1213-RapidAnt-fix-codex-title-hook-thread-fallback
  created: '2026-08-19T12:13:58+00:00'
  updated: '2026-08-19T12:14:46+00:00'
spec:
  title: Make Codex title model detection work without hook thread environment
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Fall back from CODEX_THREAD_ID to the newest active Codex thread matching
    the tmux pane working directory, because generated Codex hook subprocesses do
    not inherit the thread identifier.
  started_at: '2026-08-19T12:14:04+00:00'
  completed_at: '2026-08-19T12:14:46+00:00'
---

## Transition note (2026-08-19T12:14:04+00:00)

Reproduced empty agent after hook execution and implemented cwd-based current-thread fallback.


## Transition note (2026-08-19T12:14:46+00:00)

Focused real-tmux test covers missing CODEX_THREAD_ID/CODEX_SESSION_ID; live hook-equivalent invocation renders model and effort. Clippy, format, diff check, apply, and pk-doctor verified.


## Transition note (2026-08-19T12:14:46+00:00)

Live title remains aibox — gpt-5.6-sol low@codex when thread environment variables are absent.
