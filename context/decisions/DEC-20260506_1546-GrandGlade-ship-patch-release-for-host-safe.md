---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_1546-GrandGlade-ship-patch-release-for-host-safe
  created: '2026-05-06T15:46:11+00:00'
spec:
  title: Ship patch release for host-safe runtime smoke recovery
  state: accepted
  decision: Prepare and publish a patch release after v0.23.19 that carries the host-safe release runtime smoke harness fix from main.
  context: After stepwise recovery, v0.23.16, v0.23.17, v0.23.18, and v0.23.19 all passed when tested with a host-safe smoke harness. The published v0.23.19 tag still contains the unsafe host Phase 2 smoke behavior that streamed raw TUI control sequences to the host terminal.
  rationale: A patch release is the smallest downstream-safe way to make the release workflow itself safe for future host Phase 2 runs without changing runtime semantics beyond the smoke harness behavior.
  alternatives:
  - option: Do not release
    assessment: Leaves the public latest release with an unsafe host-side smoke harness.
  - option: Fold into a later minor release
    assessment: Delays a release-process safety fix that affects every subsequent release attempt.
  consequences: Future release-host runs should capture raw TUI output into logs rather than streaming it to the host terminal. The release remains Phase 2 gated by the user on macOS for host assets and GHCR image push.
  decided_at: '2026-05-06T15:46:11+00:00'
---
