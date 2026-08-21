---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260821_1336-CheerfulEmber-session-handover
  created: '2026-08-21T13:36:05+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-08-21T13:36:05+00:00'
  summary: Session handover — v0.34.2 fully published with theme gallery, runtime
    fixes, and corrected host Phase 2 workflow
  actor: codex
  details:
    session_date: '2026-08-21'
    current_state: 'aibox v0.34.2 is fully published: the public GitHub release contains
      Linux and Darwin archives, all four checksum sidecars, and LICENSE; the owner
      supplied successful multi-platform GHCR manifest evidence. The theme gallery/chooser,
      corrected theme definitions, Vim dim repair, tmux/Yazi pane visibility, and
      Codex question-state lifecycle fixes are merged. Host Phase 2 initially exposed
      container-local path and post-tag HEAD guard defects; both were repaired through
      PRs #438 and #439. The repository is clean on v0.x-release at 9552c203d8c2 and
      matches origin.'
    open_threads: []
    next_recommended_action: Upgrade the original derived project to v0.34.2, run
      aibox apply, and verify the Codex question marker, Vim startup, and active tmux
      pane behavior in the real downstream environment.
    branch: v0.x-release
    commit: 9552c203d8c2
    behavioral_retrospective:
    - The initial handoff incorrectly presented release-host as version-based even
      though it requires a prepared run directory; the workflow now prepares immutable
      inputs on the host and emits a repository-relative path, with regression coverage.
    - The first path fix updated release-host-prepare without reconciling the Python
      gate's exact-HEAD invariant; the gate now consistently allows a clean protected-branch
      descendant while keeping provenance and source bound exactly to the release
      tag.
    - No promised WorkItem or decision creation remains deferred; live queries found
      no in-progress or blocked WorkItems.
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.34.2
    merged_prs:
    - '#435'
    - '#436'
    - '#437'
    - '#438'
    - '#439'
    verification: Full release validation and visual E2E passed; pk-doctor previously
      reported 0 errors, 0 warnings, and 0 actionable findings; live release assets
      rechecked after host publication.
---
