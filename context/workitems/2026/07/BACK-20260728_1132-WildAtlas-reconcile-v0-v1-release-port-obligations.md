---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260728_1132-WildAtlas-reconcile-v0-v1-release-port-obligations
  created: '2026-07-28T11:32:35+00:00'
spec:
  title: Reconcile v0-to-v1 release port obligations
  state: backlog
  type: task
  priority: critical
  description: 'Release-check-state blocks the v1 alpha because seven v0 commits lack
    traceable v1 port or Not-applicable trailers: c377f5fc, 57d06354, 612138ec, f494f282,
    6f23ab7f, 4936d831, 61e92e6a. Audit each commit, port applicable source/generator
    fixes, classify context-only handovers accurately, and make scripts/check-version-line-ports.sh
    check v1 pass without weakening the gate.'
  scope: v1-alpha
---
