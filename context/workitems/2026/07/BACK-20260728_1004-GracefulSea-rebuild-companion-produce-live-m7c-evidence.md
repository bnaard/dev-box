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
  updated: '2026-08-01T07:53:21+00:00'
spec:
  title: Rebuild companion and produce candidate-bound live M7c evidence
  state: blocked
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


## Transition note (2026-08-01T07:53:21+00:00)

Blocked on host-only container recreation. Live preflight on 2026-08-01 found PID 1=sshd, kind/kubectl absent, /lib/modules absent, and Podman cgroup manager=cgroupfs. This devcontainer has no Docker/Podman client, socket, TCP API, or host SSH control path. Owner must run from the host checkout: docker compose -f .devcontainer/docker-compose.yml -f .devcontainer/docker-compose.override.yml build --no-cache --pull aibox-e2e-testrunner; then docker compose -f .devcontainer/docker-compose.yml -f .devcontainer/docker-compose.override.yml up -d --force-recreate aibox-e2e-testrunner. Candidate c22f1e4f3afc718488099c36898016f5f7357174; current debug binary sha256:e61ee0026573075474c32717f66cb470f62636646a65aa542070cfa4fc85cfa0.
