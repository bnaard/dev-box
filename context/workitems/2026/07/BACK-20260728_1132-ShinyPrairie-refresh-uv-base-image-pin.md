---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260728_1132-ShinyPrairie-refresh-uv-base-image-pin
  created: '2026-07-28T11:32:47+00:00'
spec:
  title: Refresh uv base-image pin after v1 alpha
  state: backlog
  type: task
  priority: medium
  description: Defer uv 0.11.32 to 0.11.33 from the current blocked alpha candidate
    because the same-day patch has no stated critical fix relevant to aibox and would
    widen the release scope. During the next base-image refresh, update the pin, rebuild
    the base image, and run layout, status, and Yazi runtime smoke coverage.
  scope: v1-post-alpha
---
