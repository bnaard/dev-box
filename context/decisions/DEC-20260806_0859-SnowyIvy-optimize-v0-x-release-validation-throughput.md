---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260806_0859-SnowyIvy-optimize-v0-x-release-validation-throughput
  created: '2026-08-06T08:59:01+00:00'
spec:
  title: Optimize v0.x release validation throughput
  state: accepted
  decision: 'Adopt three coordinated release optimizations: benchmark and enable an
    overlay-capable E2E companion storage path with a safe VFS fallback; isolate and
    shard resource-heavy Tier 2 tests so retries are shard-scoped; and retain cumulative,
    attempt-aware timing evidence across resumed release commands.'
  context: The v0.30.0 container-side release took about 117 minutes. Two Tier 2 attempts
    consumed 98m51s, with concurrent heavy image builds causing a timeout and the
    companion using the slow VFS storage driver.
  rationale: The changes target the measured bottlenecks while preserving the full
    compatibility gate and candidate-bound evidence.
  alternatives:
  - option: Always serialize the entire Tier 2 suite
    reason_rejected: Prevents contention but unnecessarily slows independent fast
      tests.
  - option: Remove the full addon composition test
    reason_rejected: Would weaken cross-addon compatibility coverage.
  - option: Only increase timeouts
    reason_rejected: Masks contention without reducing duration.
  consequences: Release validation becomes deterministic and resumable at shard granularity.
    Companion setup gains an overlay capability probe and must retain a compatible
    fallback. Timing reports become append-only and cumulative.
  related_workitems:
  - BACK-20260803_0713-SnappyOasis-prevent-tier2-release-e2e-contention
  decided_at: '2026-08-06T08:59:01+00:00'
---
