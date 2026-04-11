---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260411_0718-cleverFinch-process-created
  created: '2026-04-11T07:18:31+00:00'
spec:
  event_type: process.created
  timestamp: '2026-04-11T07:18:31+00:00'
  summary: Created aibox-release Process entity (PROC-20260411_0713-warmCliff-aibox-release)
  actor: claude-sonnet-4-6
  subject: PROC-20260411_0713-warmCliff-aibox-release
  subject_kind: Process
  details:
    path: context/processes/aibox-release.md
    reason: Codify the two-phase aibox release workflow so the agent never bypasses
      maintain.sh release again (v0.17.13/v0.17.14 released without binaries due to
      manual gh release create)
    related_agents_md_change: Added two anti-pattern entries for direct gh release
      create and releasing with broken cargo build
---
