---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260502_0744-SteadyPeak-runtime-resource-roadmap-followups
  created: '2026-05-02T07:44:29+00:00'
  updated: '2026-05-02T08:03:14+00:00'
spec:
  title: Implement unblocked runtime resource roadmap follow-ups
  state: done
  type: task
  priority: high
  description: 'Implement the remaining aibox runtime/resource improvements that do
    not depend on processkit gateway or schema delivery: docs, Zellij-visible resource
    status, additional optional base-image tools, doctor resource thresholds, regression
    tests, and base-image minimum contract.'
  started_at: '2026-05-02T07:44:33+00:00'
  completed_at: '2026-05-02T08:03:14+00:00'
---

## Transition note (2026-05-02T07:44:33+00:00)

Starting implementation of all unblocked follow-up items with at most two parallel agents.


## Transition note (2026-05-02T08:03:11+00:00)

Implemented non-processkit-dependent follow-ups: runtime status line, doctor thresholds, optional audio/preview tooling, docs, tests, and generated-file refresh. Full cargo test and clippy passed.


## Transition note (2026-05-02T08:03:14+00:00)

Validated with cargo test --manifest-path cli/Cargo.toml, cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings, git diff --check, aibox apply --no-container, and runtime resource/status smoke checks. No open migrations were reported by apply.
