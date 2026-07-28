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
  updated: '2026-07-28T11:18:47+00:00'
spec:
  title: Rebuild companion and produce candidate-bound live M7c evidence
  state: in-progress
  type: task
  priority: critical
  description: Rebuild the aibox E2E companion with kind, systemd PID 1, delegated
    cgroup v2 controllers, and required modules; run the exact v1.x-pre-release candidate
    through the disposable Kubernetes lifecycle; retain schema-valid candidateCommit/binarySha256-bound
    evidence; rerun release readiness. Do not publish alpha unless this and every
    other mandatory gate pass.
  started_at: '2026-07-28T11:18:47+00:00'
---

## Transition note (2026-07-28T11:18:47+00:00)

Implementation merged via PR #241 to v1.x-dev and promoted via PR #242 to v1.x-pre-release. Companion readiness is now fail-closed, M7c evidence is bound to the exact candidate binary digest, and nested repositories are excluded from owning supply-chain scans. Live evidence remains pending because the external Tier 2 companion is stale (kind absent). Host must run: docker compose -f .devcontainer/docker-compose.yml -f .devcontainer/docker-compose.override.yml up -d --build --force-recreate aibox-e2e-testrunner
