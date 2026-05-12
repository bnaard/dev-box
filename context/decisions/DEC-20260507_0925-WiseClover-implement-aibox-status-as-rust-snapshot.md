---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_0925-WiseClover-implement-aibox-status-as-rust-snapshot
  created: '2026-05-07T09:25:25+00:00'
spec:
  title: Implement aibox-status as Rust Snapshot Reader
  state: accepted
  decision: aibox-status will be implemented as a Rust executable with no Bash subprocess fanout. It will read the diagnostics sidecar's latest bounded snapshot, format plain/JSON/watch output, and degrade quickly when diagnostics are unavailable or stale.
  context: The owner accepted the diagnostics sidecar architecture and explicitly required that aibox-status become a Rust executable. The 2026-05-07 incident evidence showed thousands of bash/sh/tr/cat/awk processes created by shell-based status collection while Zellij native status was active.
  rationale: Rust direct file reads eliminate per-PID shell fanout, keep status reads cheap, and preserve one compatibility boundary for shell mode, native Zellij mode, release smoke tests, and manual debugging. Collection remains owned by the sidecar so aibox-status does not become another collector.
  alternatives:
  - option: Keep Bash aibox-status and add guards
    reason: Rejected because the incident evidence implicates shell fanout itself.
  - option: Let the Zellij plugin parse diagnostics directly
    reason: Rejected as primary path because it couples UI startup to diagnostics parsing and removes a harness-neutral status CLI boundary.
  - option: Embed collection in aibox-status Rust binary
    reason: Deferred except as a degraded fallback; collection ownership belongs in the constrained sidecar.
  consequences: Implementation must add a Rust status binary, update image/runtime seeding and tests, keep the CLI contract --plugin-json and --watch, and add release gates that prove no shell fanout remains in status paths.
  decided_at: '2026-05-07T09:25:25+00:00'
---
