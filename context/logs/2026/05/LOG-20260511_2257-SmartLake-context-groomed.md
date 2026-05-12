---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260511_2257-SmartLake-context-groomed
  created: '2026-05-11T22:57:56+00:00'
spec:
  event_type: context.groomed
  timestamp: '2026-05-11T22:57:56+00:00'
  summary: Applied local context hygiene stopgap and Yazi preview correction while waiting for processkit upstream fixes.
  actor: codex
  subject: context
  subject_kind: Repository
  details:
    removed: Superseded context/models MODEL files after matching Artifact ModelSpec records were confirmed; unmanaged .DS_Store files under context were removed.
    updated: cli/src/seed.rs directory preview now tracks direct vs inherited git status so child-only ignored/changed/added/deleted state is lowercase and does not dim the parent directory row.
    upstream_issues:
    - 39
    - 40
    - 41
---
