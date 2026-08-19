---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260819_1858-CrispLake-show-recent-v0-releases-and-hide
  created: '2026-08-19T18:58:14+00:00'
spec:
  title: Show recent v0 releases and hide the v1 documentation line
  state: accepted
  decision: The public documentation Releases selector lists the current v0 release
    and at least every release represented in the Change log. The v1.x documentation
    line is not exposed for now.
  rationale: The selector should give readers access to the releases the site publicly
    documents without advertising an incomplete v1.x line.
  consequences: The v0.34.0, v0.33.2, v0.33.1, and v0.33.0 documentation archives
    are exposed in the selector. v1.x is removed until the owner makes a later publication
    decision.
  decided_at: '2026-08-19T18:58:14+00:00'
---
