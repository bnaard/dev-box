---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_1820-DaringMelody-resolve-github-issue-133-stable-only
  created: '2026-07-23T18:20:19+00:00'
  updated: '2026-07-23T18:20:23+00:00'
spec:
  title: 'Resolve GitHub issue #133: stable-only processkit latest'
  state: in-progress
  type: bug
  priority: high
  description: 'Ensure processkit.version = "latest" selects stable releases only,
    retains explicit prerelease pins, provides prerelease opt-in in appropriate interactive/version-list
    surfaces, documents the contract, and forward-ports applicable behavior from v0.x
    to v1.x. Evidence must close GitHub issue #133 after protected-branch merges.'
  started_at: '2026-07-23T18:20:23+00:00'
---

## Transition note (2026-07-23T18:20:23+00:00)

Investigating and implementing the stable-only latest selection behavior for GitHub issue #133.
