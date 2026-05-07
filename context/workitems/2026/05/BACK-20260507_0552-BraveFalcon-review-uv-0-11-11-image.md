---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0552-BraveFalcon-review-uv-0-11-11-image
  created: '2026-05-07T05:52:32+00:00'
  labels:
    release: 0.24.0
    source: release-check-state
    deferred_dependency: uv
spec:
  title: Review uv 0.11.11 image update after v0.24.0
  state: backlog
  type: task
  priority: medium
  description: Release-check-state for aibox v0.24.0 reported the pinned uv image
    selector at ghcr.io/astral-sh/uv:0.11.10 while 0.11.11 is available. Deferred
    from v0.24.0 to keep this release focused on runtime TUI stability. Before shipping,
    review uv 0.11.11 release notes, update the pinned image selector if appropriate,
    rebuild generated devcontainers/base image surfaces, and run the runtime smoke
    plus CLI validation suite.
---
