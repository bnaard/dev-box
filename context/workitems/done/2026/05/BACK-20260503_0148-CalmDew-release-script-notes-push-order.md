---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260503_0148-CalmDew-release-script-notes-push-order
  created: '2026-05-03T01:48:15+00:00'
  labels:
    area: release
    source: session-handover
    release: 0.23.0
  updated: '2026-05-10T03:27:22+00:00'
spec:
  title: Improve aibox release script notes and push ordering
  state: done
  type: bug
  priority: medium
  description: During the v0.23.0 release, `./scripts/maintain.sh release 0.23.0`
    created the GitHub Release with auto-generated commit-list notes before there
    was a checkpoint to write curated release notes. The release script also pushed
    the tag before `main` was updated on origin, requiring a manual `git push origin
    main` after the release. Update the release flow so comprehensive curated notes
    are prepared before `gh release create`, and so the version-bump commit is pushed
    to `main` before or atomically with the tag/release creation.
  started_at: '2026-05-10T03:24:56+00:00'
  completed_at: '2026-05-10T03:27:22+00:00'
---

## Transition note (2026-05-10T03:27:22+00:00)

Implemented and merged in commit acf3d00 + merge e437b7a. scripts/maintain.sh::cmd_release now pushes main before tag and adds a notes-curation checkpoint before `gh release create`. Manual shellcheck still recommended pre-next-release.
