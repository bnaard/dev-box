---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1300-FinePlum-external-issue-created
  created: '2026-05-13T13:00:29+00:00'
spec:
  event_type: external_issue.created
  timestamp: '2026-05-13T13:00:29+00:00'
  summary: Filed upstream processkit issue for strict schema/storage migration enforcement
  actor: codex
  subject: https://github.com/projectious-work/processkit/issues/47
  subject_kind: ExternalIssue
  details:
    repo: projectious-work/processkit
    issue: 47
    title: Enforce strict schema/storage migration instead of grandfathering legacy
      vocabulary and filenames
    local_decision: DEC-20260513_1249-GrandSpruce-strictly-migrate-processkit-context-instead-of
---
