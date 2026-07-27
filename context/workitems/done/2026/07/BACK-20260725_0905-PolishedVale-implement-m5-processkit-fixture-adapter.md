---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_0905-PolishedVale-implement-m5-processkit-fixture-adapter
  created: '2026-07-25T09:05:28+00:00'
  updated: '2026-07-25T09:35:09+00:00'
spec:
  title: Implement M5 processkit protocol fixture adapter
  state: done
  type: story
  priority: high
  description: Freeze narrow processkit install request/result fixtures and error
    semantics with processkit#118, then implement availability discovery and a transport
    adapter against recorded fixtures and a fake CLI. Do not remove v0 policy or claim
    production integration until a compatible producer release exists.
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  started_at: '2026-07-25T09:07:11+00:00'
  completed_at: '2026-07-25T09:35:09+00:00'
---

## Transition note (2026-07-25T09:07:11+00:00)

Start producer-gated M5 fixture adapter groundwork.


## Transition note (2026-07-25T09:14:28+00:00)

Provisional fixture contract and fake CLI transport adapter implemented in b5419c68; full tests and clippy pass. Production integration remains explicitly gated on processkit#118.


## Transition note (2026-07-25T09:35:09+00:00)

Completed the scoped producer-gated groundwork: provisional v1alpha1 protocol types, fake CLI adapter, fixtures, retry/error/provenance semantics, and secret-canary tests. Production wiring remains blocked on processkit #118 and is not claimed by this WorkItem.
