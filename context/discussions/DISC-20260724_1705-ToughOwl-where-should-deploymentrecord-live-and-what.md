---
apiVersion: processkit.projectious.work/v2
kind: Discussion
metadata:
  id: DISC-20260724_1705-ToughOwl-where-should-deploymentrecord-live-and-what
  created: '2026-07-24T17:05:41+00:00'
  updated: '2026-07-24T17:17:52+00:00'
spec:
  question: Where should DeploymentRecord live, and what rollback, DNS, and ingress
    boundaries should aibox v1 guarantee?
  state: resolved
  opened_at: '2026-07-24T17:05:41+00:00'
  related:
  - DISC-20260724_1653-BuoyantOtter-which-refinements-and-decisions-are-required
  outcomes:
  - DEC-20260724_1717-WildLynx-define-aibox-v1-deployment-state-and
  - DEC-20260724_1705-LuckyPearl-develop-and-support-aibox-v1-as
  closed_at: '2026-07-24T17:17:52+00:00'
---

## Owner refinement recorded

Accepted decision: `DEC-20260724_1705-LuckyPearl-develop-and-support-aibox-v1-as`.

The owner selected feature branches inside the independent v1.x line; concurrent v0.x and v1.x support; patch-only v0 maintenance after stable v1; adoption-driven v0 retirement; apply-only `up` in stable v1; explicit opt-in image builds; Kubernetes ingress and DNS; and processkit delegation as a stable-v1 release gate.

## Remaining questions

1. DeploymentRecord authority: project-local, target-side, or dual-written?
2. Rollback meaning: reuse v0 on a v1-migrated project, redeploy through v0, or keep separate v0/v1 deployments during evaluation?
3. DNS: manage records only in pre-existing zones, and through which initial provider/API abstraction?
4. Ingress: consume existing IngressClass/GatewayClass only, or install controllers?

Implementation remains paused pending refinement.
