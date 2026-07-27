---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260724_1705-LuckyPearl-develop-and-support-aibox-v1-as
  created: '2026-07-24T17:05:31+00:00'
spec:
  title: Develop and support aibox v1 as an independent version line
  state: accepted
  decision: Develop v1 work on feature branches rooted in the independent v1.x line.
    Continue supporting v0.x alongside stable v1; when v1 stabilizes, v0.x enters
    patch-only maintenance and is removed only after adoption and migration evidence
    justify it. Stable v1 makes `aibox up` apply-only, image builds explicit and opt-in,
    includes ingress and DNS reconciliation in Kubernetes scope, and requires processkit
    v1 delegation before release.
  context: 'Owner refinement of the draft workplan derived from projectious-work/aibox
    issue #179.'
  rationale: This avoids a big-bang replacement, protects existing projects, keeps
    v1 architecture independent from v0 maintenance, separates deployment from connection,
    prevents hidden remote builds, and makes the intended Kubernetes and processkit
    boundaries explicit.
  alternatives:
  - option: Replace v0.x when v1 becomes stable
    reason_rejected: Would force migration before adoption and rollback evidence is
      sufficient.
  - option: Block Kubernetes development on processkit delegation
    reason_rejected: Processkit is required for release but should not block independent
      backend development.
  - option: Keep implicit attach and implicit image builds
    reason_rejected: Conflates lifecycle concerns and hides mutation during deployment.
  consequences: The roadmap needs parallel release-line support, deliberate patch
    porting, adoption-based v0 retirement gates, explicit migration tooling, ingress/DNS
    ownership contracts, and a hard stable-v1 gate on processkit protocol integration.
  decided_at: '2026-07-24T17:05:31+00:00'
---
