---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260515_0712-TrustyRiver-post-release-dependency-drift
  created: '2026-05-15T07:12:59+00:00'
  labels:
    source: pk-wrapup
    release: v0.26.4
    category: maintenance
spec:
  title: Review post-v0.26.4 dependency drift
  state: backlog
  type: task
  priority: medium
  description: 'During pk-wrapup after the completed v0.26.4 release, `./scripts/maintain.sh
    release-check-state` reported routine dependency drift: uv image is pinned at
    `ghcr.io/astral-sh/uv:0.11.11` while latest is `0.11.14`; `cargo update --dry-run`
    would update `clap_complete` 4.6.4 -> 4.6.5, `filetime` 0.2.28 -> 0.2.29, and
    `winnow` 1.0.2 -> 1.0.3; harness latest channels should be reviewed during the
    next maintenance pass. This was not a v0.26.4 release blocker because the release
    and host-side Phase 2 were already complete, but it should be resolved or explicitly
    deferred before the next patch release.'
  scope: aibox
---
