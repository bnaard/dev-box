---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260425_0958-ThriftyEagle-hold-90-day-focus
  created: '2026-04-25T09:58:46+00:00'
spec:
  title: Hold 90-day focus items pending processkit v0.21.0 release
  state: accepted
  decision: Defer start of 90-day focus items (security review, pre-install three-way,
    version upgrade flows, preview companion, process retrospective) until processkit
    v0.21.0 is released and integrated. Reassess priorities and blockers in light
    of new processkit features/changes before commencing work.
  context: 'Backlog grooming complete with 5-item 90-day focus established (DEC-20260425_0953).
    Current repository state: stable, 0 pk-doctor errors, all MCP permissions feature
    complete, all tests passing. v0.21.0 release expected to bring processkit enhancements
    that may affect scope/priority of planned work.'
  rationale: Waiting for v0.21.0 allows us to integrate upstream changes first and
    reassess the 90-day plan in context of new capabilities. Prevents rework if v0.21.0
    addresses any of the planned items or changes architectural assumptions underlying
    the work.
  related_workitems:
  - BACK-20260411_0000-JollyWren-security-review-cli-input
  - BACK-20260411_0000-DeepComet-pre-install-three-way
  - BACK-20260411_0000-LoyalSeal-version-upgrade-flows-review
  - BACK-20260411_0000-CoolBear-preview-companion-design-review
  - BACK-20260411_0000-AmberWren-process-model-retrospective-competitive
  decided_at: '2026-04-25T09:58:46+00:00'
---
