---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_1003-MightyVale-complete-m5-production-processkit-protocol-delegation
  created: '2026-07-25T10:03:54+00:00'
  labels:
    milestone: M5-production
    line: v1.x
    blocked_by: processkit#118
  updated: '2026-07-25T10:51:11+00:00'
spec:
  title: Complete M5 production processkit protocol delegation
  state: blocked
  type: story
  priority: high
  description: 'When processkit #118 publishes a compatible released CLI protocol,
    replace provisional fixtures with released schemas, invoke the real CLI, prove
    lifecycle and parity behavior, and remove duplicated aibox policy only after migration
    and rollback gates pass. Until then, retain and test the explicit external gate.'
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  started_at: '2026-07-25T10:50:54+00:00'
---

## Transition note (2026-07-25T10:50:54+00:00)

Provisional M5 adapter remains tested; production completion is externally blocked until processkit issue #118 publishes the released producer CLI protocol.


## Transition note (2026-07-25T10:51:11+00:00)

Blocked on open upstream processkit issue #118 and a compatible released producer CLI protocol; fixture-only integration remains deliberately non-production.
