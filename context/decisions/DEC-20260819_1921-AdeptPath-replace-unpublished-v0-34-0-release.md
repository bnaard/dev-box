---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260819_1921-AdeptPath-replace-unpublished-v0-34-0-release
  created: '2026-08-19T19:21:29+00:00'
spec:
  title: Replace unpublished v0.34.0 release provenance
  state: accepted
  decision: Remove the existing v0.34.0 GitHub release and tag before host publication,
    implement the remaining tmux title-refresh correction, then recreate v0.34.0 from
    the latest v0.x release-line commit with rebuilt Linux assets and redeployed documentation.
  context: The first v0.34.0 tag predates the final documentation corrections and
    host Phase 2 has not run. The owner explicitly authorized discarding the existing
    tag and Linux binaries.
  rationale: Recreating the not-yet-host-published release keeps the tag, source,
    Linux binaries, documentation, and later host artifacts aligned under one version.
  alternatives:
  - option: Publish v0.34.1
    reason: Avoids moving a tag but introduces another version before v0.34.0 host
      publication.
  - option: Proceed with existing v0.34.0
    reason: Would leave final docs and tmux refresh correction outside tagged source.
  consequences: The existing GitHub release assets and tag are intentionally replaced.
    Any consumers of the short-lived original tag would observe the tag move, so provenance
    before deletion must be captured.
  decided_at: '2026-08-19T19:21:29+00:00'
---
