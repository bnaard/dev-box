---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_0905-AmpleLeaf-implement-m6-kubernetes-validation-plan
  created: '2026-07-25T09:05:38+00:00'
  updated: '2026-07-25T09:35:46+00:00'
spec:
  title: Implement M6 Kubernetes validation and planning
  state: done
  type: story
  priority: high
  description: Implement typed Kubernetes target validation, explicit context/namespace
    authorization, non-mutating capability discovery, and deterministic golden manifest
    planning from the same canonical fleet used by Compose. Include ownership labels,
    ingress/DNS capability placeholders, credential references only, and no infrastructure
    provisioning.
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  started_at: '2026-07-25T09:16:42+00:00'
  completed_at: '2026-07-25T09:35:46+00:00'
---

## Transition note (2026-07-25T09:16:42+00:00)

Start M6 Kubernetes validation and planning after M3 backend contracts completed.


## Transition note (2026-07-25T09:31:29+00:00)

Non-mutating Kubernetes target validation, discovery abstraction, deterministic golden rendering, ownership labels, and constrained ingress/DNS intent implemented in 3adf867c; full tests and clippy pass.


## Transition note (2026-07-25T09:35:46+00:00)

Integrated pure Kubernetes planning with explicit context/namespace validation, read-only discovery, immutable images, ownership labels, constrained existing ingress/DNS intent, golden output, and cross-backend tests; combined validation passed.
