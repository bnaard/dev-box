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
  updated: '2026-07-28T06:01:36+00:00'
spec:
  title: Complete M5 production processkit protocol delegation
  state: done
  type: story
  priority: high
  description: 'When processkit #118 publishes a compatible released CLI protocol,
    replace provisional fixtures with released schemas, invoke the real CLI, prove
    lifecycle and parity behavior, and remove duplicated aibox policy only after migration
    and rollback gates pass. Until then, retain and test the explicit external gate.'
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  started_at: '2026-07-25T10:50:54+00:00'
  completed_at: '2026-07-28T06:01:36+00:00'
---

## Transition note (2026-07-25T10:50:54+00:00)

Provisional M5 adapter remains tested; production completion is externally blocked until processkit issue #118 publishes the released producer CLI protocol.


## Transition note (2026-07-25T10:51:11+00:00)

Blocked on open upstream processkit issue #118 and a compatible released producer CLI protocol; fixture-only integration remains deliberately non-production.


## Transition note (2026-07-25T21:24:57+00:00)

Processkit reported in aibox Discussion #186 that PR #123 at d800953 completes the installer/v1alpha1 producer implementation and is ready for consumer compatibility testing. Resume M5 against that head while retaining the v0 fallback and stable-release gates.


## Transition note (2026-07-28T06:01:36+00:00)

processkit v1.0.0-alpha.3 publishes the installer/v1alpha1 protocol and signed native assets. aibox commit 210abe11 pins the release and the public consumer gate passed lifecycle plus interruption recovery, retry refusal, lock, secret-safety, v0 coexistence, and uninstall evidence.


## Transition note (2026-07-28T06:01:36+00:00)

Production protocol delegation acceptance evidence is complete; stable-v1 remains governed by separate M7, parity, and release gates.
