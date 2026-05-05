---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260505_2053-RapidWren-review-deferred-dependency-updates
  created: '2026-05-05T20:53:43+00:00'
  labels:
    release: v0.23.16
    source: release-check-state
    deferred: true
spec:
  title: Review and apply deferred dependency updates from v0.23.16 release check
  state: backlog
  type: task
  priority: medium
  description: 'The v0.23.16 release-check-state report detected non-blocking dependency
    drift that was intentionally deferred from the urgent compatibility patch. Review
    and decide/update separately: Zellij 0.44.1 -> 0.44.2, Yazi 26.1.22 -> 26.5.6,
    floating uv/Node/Debian inputs, unpinned AI harness installer/package surfaces,
    and Cargo.lock dry-run crate updates. Validate runtime layouts, Yazi config/plugin
    compatibility, native Zellij key/status plugin behavior, and generated container
    images before shipping any bumps.'
  scope: aibox release maintenance
---
