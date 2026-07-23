---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_1856-CuriousDeer-release-v0287-infrastructure-checksum-fix
  created: '2026-07-23T18:56:34+00:00'
  updated: '2026-07-23T19:19:57+00:00'
spec:
  title: Release aibox v0.28.7 infrastructure checksum verification fix
  state: done
  type: bug
  priority: high
  description: Prepare and publish the v0.28.7 patch release containing the OpenTofu
    and Packer archive checksum verification fix. Validate the release line, publish
    repository-side Linux assets and docs, then hand off macOS/GHCR Phase 2 to the
    owner.
  started_at: '2026-07-23T18:56:36+00:00'
  completed_at: '2026-07-23T19:19:57+00:00'
---

## Transition note (2026-07-23T18:56:36+00:00)

Release preparation started on v0.x-release.


## Transition note (2026-07-23T19:10:56+00:00)

Repository-side v0.28.7 release published and verified: GitHub release has Linux archives/checksums and LICENSE; docs deployed. Awaiting owner-run macOS/GHCR Phase 2.


## Transition note (2026-07-23T19:19:57+00:00)

Host Phase 2 complete. Verified all Linux and macOS archives/checksums on GitHub Release v0.28.7, generated runtime synchronized, v0 port gate clear, and release promoted to v0.x-dev and main via PRs #149 and #150.
