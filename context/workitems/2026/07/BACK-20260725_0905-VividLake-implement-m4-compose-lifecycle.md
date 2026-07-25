---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_0905-VividLake-implement-m4-compose-lifecycle
  created: '2026-07-25T09:05:21+00:00'
spec:
  title: Implement M4 Compose deployment lifecycle
  state: backlog
  type: story
  priority: high
  description: Implement atomic and locked DeploymentRecord storage, immutable image
    resolution, idempotent Compose apply, backend-neutral status and logs, drift classification,
    ownership-guarded destroy, compose-exec connection targets, and recovery/concurrency/secret-absence
    tests.
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
---
