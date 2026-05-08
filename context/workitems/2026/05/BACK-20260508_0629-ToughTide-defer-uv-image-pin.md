---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_0629-ToughTide-defer-uv-image-pin
  created: '2026-05-08T06:29:12+00:00'
spec:
  title: Review and defer uv image pin update from release-check-state
  state: backlog
  type: chore
  priority: medium
  description: Release-check-state on 2026-05-08 reported `ghcr.io/astral-sh/uv:0.11.10`
    with update available to `0.11.11`. Deferred from v0.25.2 release. Validate uv
    release notes, bump the pinned tag in image build inputs, rebuild base image,
    and rerun release validation gates including runtime visual status checks.
---
