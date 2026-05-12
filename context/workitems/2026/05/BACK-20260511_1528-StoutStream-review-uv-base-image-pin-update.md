---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260511_1528-StoutStream-review-uv-base-image-pin-update
  created: '2026-05-11T15:28:32+00:00'
  labels:
    source: release-check-state
    release: 0.25.8
    component: base-image
    deferred: true
spec:
  title: Review uv base image pin update to 0.11.13
  state: backlog
  type: task
  priority: medium
  description: Phase 0 release state for v0.25.8 found `ghcr.io/astral-sh/uv:0.11.11` with latest `0.11.13`. Deferred from this patch release because the current change set is focused on tmux/runtime logging and the uv image bump requires base-image rebuild review plus runtime smoke validation.
---
