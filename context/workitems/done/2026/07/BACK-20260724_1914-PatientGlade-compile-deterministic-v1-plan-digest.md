---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260724_1914-PatientGlade-compile-deterministic-v1-plan-digest
  created: '2026-07-24T19:14:46+00:00'
  updated: '2026-07-25T09:07:04+00:00'
spec:
  title: Compile deterministic v1 canonical plan and digest
  state: done
  type: story
  priority: high
  description: 'Implement a pure compiler from typed v1 configuration into canonical
    WorkspaceImageSpec, WorkspaceFleetSpec, DeploymentTarget and normalized deployment
    plan. Define stable canonicalization and SHA-256 desired-spec digest, explicit
    opt-in image-build planning, deterministic cross-platform fixtures, invalid-input
    preflight tests, and no backend mutation. Refs GitHub #179 and ART-20260724_1653-FairHare-draft-aibox-v1-image-and-deployment.'
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  scope: v1.x
  started_at: '2026-07-24T19:14:55+00:00'
  completed_at: '2026-07-25T09:07:04+00:00'
---

## Transition note (2026-07-24T19:14:55+00:00)

Starting in an isolated v1.x worktree from pushed branch agent/v1-m0-m1-foundation.


## Transition note (2026-07-24T19:25:53+00:00)

Implemented and pushed on origin/agent/v1-m2-config-compiler. Commits 57e66c94 and integration adapter 62bec040. Pure compiler normalizes canonical contracts, emits explicit-build-only ordered actions, calculates canonical JSON SHA-256 desired-spec digest, and performs no discovery or mutation. Full validation: 1072 unit tests, 88 Tier-1 E2E passed (1 ignored), 30 integration tests, fmt, and zero-warning clippy.


## Transition note (2026-07-25T09:07:04+00:00)

Validated and merged into v1.x-dev through PR #183.
