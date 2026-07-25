---
apiVersion: processkit.projectious.work/v2
kind: Discussion
metadata:
  id: DISC-20260724_1653-BuoyantOtter-which-refinements-and-decisions-are-required
  created: '2026-07-24T16:53:09+00:00'
  updated: '2026-07-24T17:17:52+00:00'
spec:
  question: Which refinements and decisions are required before accepting the aibox
    v1 orchestration workplan and starting implementation?
  state: resolved
  opened_at: '2026-07-24T16:53:09+00:00'
  outcomes:
  - DEC-20260724_1717-WildLynx-define-aibox-v1-deployment-state-and
  - DEC-20260724_1705-LuckyPearl-develop-and-support-aibox-v1-as
  closed_at: '2026-07-24T17:17:52+00:00'
---

## Context

- Source briefing: https://github.com/projectious-work/aibox/issues/179
- Draft plan artifact: ART-20260724_1653-FairHare-draft-aibox-v1-image-and-deployment
- Plan document: `docs/v1-workplan.md`
- Status: draft for owner review; implementation is explicitly paused.

## Proposed sequencing

Contracts and inventory first, then the canonical configuration compiler and a Compose vertical slice. Processkit v1 protocol integration and Kubernetes planning proceed as separate lanes after their prerequisites; Compose is not blocked on either processkit or ainfra-templates.

## Decisions requested from the owner

1. v1 branch strategy.
2. DeploymentRecord authority and recovery model.
3. `aibox up` implicit-attach compatibility window.
4. Whether deploy may implicitly build an image.
5. Initial Kubernetes resource scope.
6. v0 rollback boundary.
7. Whether processkit delegation must precede Kubernetes lifecycle work or only the v1 release gate.

No implementation action should begin until these are refined and accepted.
