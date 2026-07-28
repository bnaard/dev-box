---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260728_0947-ReadyOrchard-implement-issue-236-v1-alpha-stabilization
  created: '2026-07-28T09:47:13+00:00'
  labels:
    github_issue: '236'
    release_line: v1
    phase: alpha-stabilization
  updated: '2026-07-28T09:47:18+00:00'
spec:
  title: Implement issue 236 v1 alpha stabilization blockers
  state: in-progress
  type: bug
  priority: critical
  description: 'Implement the approved Phase 0 slice from GitHub issue #236: typed
    M7c evidence producer/parser/schema parity; Kubernetes destroy complete ownership
    validation before mutation; Kubernetes apply workload-first/network-last ordering
    with partial-failure tests; reconcile v1.x-dev and v1.x-pre-release. Live evidence
    and alpha publication remain fail-closed follow-on gates.'
  started_at: '2026-07-28T09:47:18+00:00'
---

## Transition note (2026-07-28T09:47:18+00:00)

Implementation started from issue #236 using three routed parallel agent worktrees for evidence parity, destroy safety, and apply ordering; central integration owns branch reconciliation and release gating.
