---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_1102-NimblePeak-session-handover
  created: '2026-05-12T11:02:16+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-12T11:01:09Z'
  summary: Session handover — aibox v0.25.10 fully released with processkit v0.26.2
    integrated
  actor: codex
  subject: aibox-v0.25.10-release
  subject_kind: Release
  details:
    session_date: '2026-05-12'
    current_state: 'processkit v0.26.2 was integrated as the default processkit version.
      aibox v0.25.10 was released: Linux release assets and checksum sidecars were
      uploaded, docs deployed, and the user confirmed host-side macOS binaries, GHCR
      images, runtime smoke, and generated runtime refresh are complete. main is at
      595f738, the host-side generated runtime refresh commit. pk-doctor was clean
      during the release flow: 0 errors, 0 warnings, 0 pending migrations.'
    open_threads:
    - 'Working tree has a small uncommitted aibox.toml generated-comment cleanup:
      one container.image comment wording change, removal of stale yazi-omp commented
      addon block, and a harness-order comment wording change.'
    - processkit release-audit still reports historical live context issues as ERROR
      even though pk-doctor reports them as grandfathered INFO; this did not block
      the aibox release path, but the mismatch remains worth upstream alignment.
    - Release script skipped opt-in full visual E2E by default; regular visual/runtime/keybinding
      tests and Tier 2 SSH companion E2E passed.
    next_recommended_action: Decide whether to commit the small aibox.toml generated-comment
      cleanup or leave it for the next config-refresh pass, then run pk-resume and
      verify the live post-rebuild tmux statusline after the user rebuilds the container.
    branch: main
    commit: 595f738
    uncommitted_changes:
    - aibox.toml
    stash: none
    validation:
    - 'pk-doctor clean in release flow: 0 errors, 0 warnings, 0 pending migrations'
    - cargo fmt passed
    - cargo clippy --all-targets -- -D warnings passed
    - 'cargo test passed: 902 unit, 90 E2E with 1 ignored, 29 integration'
    - 'Tier 2 SSH companion E2E passed: 128 passed, 7 ignored'
    - cargo audit clean
    - GitHub release v0.25.10 verified with Linux binaries and .sha256 sidecars
    - 'User confirmed host-side release steps complete: macOS binaries, GHCR images,
      runtime smoke, generated runtime refresh'
    behavioral_retrospective:
    - The first release attempt hit the known compat-table hard gate after the version
      bump. I added the 0.25.10 compat row and amended it into the release bump commit
      before rerunning the release successfully.
    - MCP WorkItem query tools returned 'Unexpected response type' during wrapup,
      so open WorkItems could not be enumerated through the intended index-management
      path in this turn.
---
