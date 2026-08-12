---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260812_1704-ClearJewel-retire-the-rust-based-aibox-v1
  created: '2026-08-12T17:04:29+00:00'
spec:
  title: Retire the Rust-based aibox v1 line in favor of a future Python specification
  state: accepted
  decision: Deactivate the existing Rust-based aibox v1 orchestration epic, its milestone
    work, and candidate-bound v1 release evidence work. Do not extend, release, or
    treat that implementation as the basis for the next v1. The next v1 will be designed
    in Python from a forthcoming detailed internal specification; no implementation
    begins until that specification is available and accepted.
  context: The existing v1.x work implemented a Rust orchestration architecture and
    Kubernetes release plan. The owner has declared that direction obsolete and intends
    to restart v1 around Python with a detailed internal specification.
  rationale: Keeping obsolete v1 work active would misroute agents, preserve irrelevant
    release gates, and encourage incremental work on an architecture that will be
    replaced. Retaining completed artifacts and history while cancelling active tracking
    preserves auditability without presenting it as current product direction.
  alternatives:
  - option: Continue the Rust v1 line until the Python specification arrives
    rejected_because: Would spend effort on an architecture already declared obsolete
      and create migration pressure for the replacement.
  - option: Delete the old v1 history and artifacts
    rejected_because: Would destroy useful provenance; cancellation and an explicit
      decision make the supersession clear while retaining the record.
  - option: Create the Python v1 epic immediately
    rejected_because: The authoritative internal specification has not arrived, so
      scope and acceptance criteria would be invented.
  consequences: 'All active and blocked Rust-v1 WorkItems are cancelled; completed
    items remain historical and must not be treated as current direction. GitHub issue
    #299 and other Rust-v1 readiness tracking should be closed as obsolete. The v1.x
    branches and artifacts remain historical unless separately authorized for archival
    or deletion. A new Python-v1 epic will be created only from the forthcoming internal
    specification.'
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  related_workitems:
  - BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  - BACK-20260725_1003-GiftedBlossom-implement-m7c-disposable-cluster-e2e-and
  - BACK-20260728_1004-GracefulSea-rebuild-companion-produce-live-m7c-evidence
  decided_at: '2026-08-12T17:04:29+00:00'
---
