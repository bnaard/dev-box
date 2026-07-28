---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260728_1004-GracefulSea-rebuild-companion-produce-live-m7c-evidence
  created: '2026-07-28T10:04:26+00:00'
  labels:
    github_issue: '236'
    epic: '179'
    release_line: v1
    gate: m7c-live
spec:
  title: Rebuild companion and produce candidate-bound live M7c evidence
  state: backlog
  type: task
  priority: critical
  description: Rebuild the aibox E2E companion with kind, systemd PID 1, delegated
    cgroup v2 controllers, and required modules; run the exact v1.x-pre-release candidate
    through the disposable Kubernetes lifecycle; retain schema-valid candidateCommit/binarySha256-bound
    evidence; rerun release readiness. Do not publish alpha unless this and every
    other mandatory gate pass.
---
