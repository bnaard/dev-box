---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260813_1636-TidyLynx-default-release-host-to-safe-cache
  created: '2026-08-13T16:36:19+00:00'
spec:
  title: Default release-host to safe cache reuse with authenticated retry
  state: accepted
  decision: Release-host will reuse safe content-addressed build caches by default,
    retain fresh runtime and security validation, and support scoped retries only
    when completed evidence is cryptographically bound to the same immutable candidate
    and step inputs.
  context: Repeated macOS Phase 2 release runs rebuild all expensive surfaces after
    a late failure, even when prior outputs remain valid.
  rationale: This reduces host duration while preserving immutable candidate provenance
    and fresh evidence for changed or dependent surfaces.
  consequences: Introduce an explicit cold-cache mode, persistent isolated caches,
    step fingerprints and retry validation; unknown impact remains fail-safe.
  related_workitems:
  - BACK-20260813_1636-StableArch-optimize-release-host-cache-retry
  decided_at: '2026-08-13T16:36:19+00:00'
---
