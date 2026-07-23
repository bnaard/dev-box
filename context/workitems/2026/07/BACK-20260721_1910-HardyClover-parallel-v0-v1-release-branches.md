---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260721_1910-HardyClover-parallel-v0-v1-release-branches
  created: '2026-07-21T19:10:09+00:00'
  labels:
    github_issue: 83
    area: release-engineering
    version-lines:
    - v0
    - v1
  updated: '2026-07-21T19:12:39+00:00'
spec:
  title: Implement parallel v0 maintenance and v1 prerelease branching strategy
  state: in-progress
  type: epic
  priority: high
  description: 'Implement GitHub issue #83: establish protected version-line development
    and release branches, make integration branches the tag authorities, adapt release
    automation and documentation for v0 maintenance/hotfix releases and v1 prereleases,
    and preserve main as the published-history integration branch. Sequence branch
    creation/protection and release-script changes so existing v0.28.x releases remain
    safe.'
  scope: release-engineering
  started_at: '2026-07-21T19:12:39+00:00'
---

## Transition note (2026-07-21T19:12:39+00:00)

Branch topology implementation started. Creating v0.x and v1.x development/release authority branches from the agreed stable and main baselines.
