---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1203-SteadyArch-render-tmux-agent-model-effort
  created: '2026-08-19T12:03:49+00:00'
  updated: '2026-08-19T12:08:15+00:00'
spec:
  title: Render tmux agent title from runtime model and effort
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Replace processkit interlocutor fallback with harness runtime model
    metadata, add basename/full agent-style configuration, and render Codex model
    plus reasoning effort from current thread state.
  started_at: '2026-08-19T12:03:54+00:00'
  completed_at: '2026-08-19T12:08:15+00:00'
---

## Transition note (2026-08-19T12:03:54+00:00)

Confirmed Codex exposes CODEX_THREAD_ID and stores the current model and reasoning_effort in its versioned state SQLite database; implementation started.


## Transition note (2026-08-19T12:08:15+00:00)

Implemented agent-style basename/full rendering. Codex resolves current model and reasoning effort from its thread state via CODEX_THREAD_ID; explicit helper/environment values remain supported. Focused real-tmux tests, clippy, format, Hugo, live apply, and live title verified.


## Transition note (2026-08-19T12:08:15+00:00)

Live full style renders aibox — gpt-5.6-sol low@codex; basename coverage renders gpt-5.6-sol.
