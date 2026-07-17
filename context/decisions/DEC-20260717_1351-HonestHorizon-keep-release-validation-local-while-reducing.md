---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260717_1351-HonestHorizon-keep-release-validation-local-while-reducing
  created: '2026-07-17T13:51:08+00:00'
spec:
  title: Keep release validation local while reducing release latency
  state: accepted
  decision: All release preparation, testing, artifact builds, container image work,
    validation, and publication remain local. GitHub Actions will not be introduced.
    Release speed will be improved through local parallel orchestration, SHA-bound
    reusable artifacts, targeted cache-preserving cleanup, isolated companion test
    shards, readiness probes, and host-phase build and smoke optimization without
    materially reducing test coverage.
  context: The current release surface is intentionally broad but sequential execution,
    repeated test binaries, destructive cache pruning, fixed waits, and sequential
    cross-target builds make releases too slow.
  rationale: This preserves owner control and the complete local testing surface while
    attacking orchestration and cache inefficiencies rather than deleting gates.
  alternatives:
  - option: GitHub Actions matrix
    status: rejected
    reason: Owner explicitly requires all release work to run locally.
  - option: Reduce release test coverage
    status: rejected
    reason: Speed should come from isolation, caching, reuse, and parallelism rather
      than removing meaningful coverage.
  consequences: The local release tooling must record timings and artifact provenance,
    prevent stale artifact reuse, manage concurrent jobs and failures safely, and
    keep publication blocked until every required local gate for the exact commit
    succeeds.
  decided_at: '2026-07-17T13:51:08+00:00'
---
