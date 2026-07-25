---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_0905-AmpleLeaf-implement-m6-kubernetes-validation-plan
  created: '2026-07-25T09:05:38+00:00'
spec:
  title: Implement M6 Kubernetes validation and planning
  state: backlog
  type: story
  priority: high
  description: Implement typed Kubernetes target validation, explicit context/namespace
    authorization, non-mutating capability discovery, and deterministic golden manifest
    planning from the same canonical fleet used by Compose. Include ownership labels,
    ingress/DNS capability placeholders, credential references only, and no infrastructure
    provisioning.
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
---
