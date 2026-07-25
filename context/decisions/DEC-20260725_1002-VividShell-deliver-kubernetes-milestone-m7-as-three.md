---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260725_1002-VividShell-deliver-kubernetes-milestone-m7-as-three
  created: '2026-07-25T10:02:03+00:00'
spec:
  title: Deliver Kubernetes milestone M7 as three reviewable increments
  state: accepted
  decision: Implement M7 as M7a for Kubernetes apply, records, status, logs, and guarded
    destroy; M7b for exec, port-forward, ingress, and DNS reconciliation; and M7c
    for disposable-cluster end-to-end validation and recovery hardening.
  context: M0 through M4 and M6 are merged into v1.x-dev. The next milestone must
    add Kubernetes mutation safely while keeping ownership, infrastructure boundaries,
    and review scope explicit.
  rationale: The split follows dependency order, keeps mutation and ownership review
    focused, and makes real-cluster validation a required M7 completion gate rather
    than deferred release work.
  consequences: M7 is not complete until M7c passes. Aibox may reconcile deployment-owned
    ingress and DNS records only against existing classes and zones, and continues
    not to provision clusters, controllers, zones, networks, or identities.
  related_workitems:
  - BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  decided_at: '2026-07-25T10:02:03+00:00'
---
