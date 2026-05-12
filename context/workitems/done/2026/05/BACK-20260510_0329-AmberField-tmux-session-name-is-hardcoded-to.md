---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0329-AmberField-tmux-session-name-is-hardcoded-to
  created: '2026-05-10T03:29:22+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux
  updated: '2026-05-10T07:14:20+00:00'
spec:
  title: "tmux session name is hardcoded to 'aibox' \u2014 should derive from project name"
  state: done
  type: bug
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:14:06+00:00'
  completed_at: '2026-05-10T07:14:20+00:00'
---

## Transition note (2026-05-10T07:14:20+00:00)

Implemented and merged in commit f39302c + merge 59d7e0d. New resolve_tmux_session_name + sanitize_tmux_session_name in cli/src/config.rs; wired through sync_grouped_sections so all callers pick it up. 9 new tests pass. Schema field is [aibox] project_name (not [project] name as the WorkItem assumed); aibox dogfood unchanged.
