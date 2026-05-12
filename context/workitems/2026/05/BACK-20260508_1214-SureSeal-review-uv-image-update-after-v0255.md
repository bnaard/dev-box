---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1214-SureSeal-review-uv-image-update-after-v0255
  created: '2026-05-08T12:14:07+00:00'
  labels:
    release: 0.25.5
    dependency: uv
    deferred: true
spec:
  title: Review uv image update after v0.25.5
  state: backlog
  type: task
  priority: medium
  description: 'Release-state check for aibox v0.25.5 reported the pinned uv image at ghcr.io/astral-sh/uv:0.11.10 with 0.11.11 available. Deferred from v0.25.5 because the patch release is scoped to tmux runtime attach/session fixes. Validation required before shipping later: inspect uv 0.11.11 release notes, update image pin if appropriate, rebuild affected images, and rerun generated runtime plus companion E2E coverage.'
  scope: release-dependency-followup
---
