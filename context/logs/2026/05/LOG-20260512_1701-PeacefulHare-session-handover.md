---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_1701-PeacefulHare-session-handover
  created: '2026-05-12T17:01:33+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-12T17:00:16Z'
  summary: Session handover - v0.25.11 release completed on Linux and host sides
  actor: codex
  details:
    session_date: '2026-05-12'
    current_state: 'aibox v0.25.11 is complete end to end. Linux-side release created
      GitHub release v0.25.11 with Linux binaries and checksum assets, deployed documentation
      to gh-pages, and wrote the host-side prompt. The user then completed host Phase
      2: macOS binaries were uploaded, container images pushed to GHCR, runtime smoke
      passed with logs under dist/release-smoke/v0.25.11/, and generated runtime refresh
      was committed as d4f401db. The repository is clean on main at d4f401db.'
    open_threads:
    - No in-progress or blocked WorkItems were returned by processkit.
    - No pending migrations were returned by migration-management.
    - Tier 2 SSH companion E2E and opt-in visual E2E were intentionally skipped during
      the urgent Linux release; host runtime smoke passed afterward.
    - Release doctor and release state artifacts remain under dist/ for reference;
      runtime smoke logs are under dist/release-smoke/v0.25.11/.
    next_recommended_action: Start the next session with pk-resume and verify whether
      any post-release reports appeared for v0.25.11. If there are no incidents, the
      highest-value follow-up is to run the skipped companion/visual E2E suite when
      time permits, using the SSH companion path rather than Docker/Podman from the
      main devcontainer.
    branch: main
    commit: d4f401db
    release:
      version: 0.25.11
      tag: v0.25.11
      url: https://github.com/projectious-work/aibox/releases/tag/v0.25.11
      assets_verified:
      - aibox-v0.25.11-aarch64-apple-darwin.tar.gz
      - aibox-v0.25.11-aarch64-apple-darwin.tar.gz.sha256
      - aibox-v0.25.11-aarch64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.11-aarch64-unknown-linux-gnu.tar.gz.sha256
      - aibox-v0.25.11-x86_64-apple-darwin.tar.gz
      - aibox-v0.25.11-x86_64-apple-darwin.tar.gz.sha256
      - aibox-v0.25.11-x86_64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.11-x86_64-unknown-linux-gnu.tar.gz.sha256
      linux_release_commit: 03eeffdf
      host_refresh_commit: d4f401db
    git:
      working_tree: clean
      stash: none
      recent_commits:
      - 'd4f401db chore: refresh generated runtime for v0.25.11'
      - '03eeffdf test: update tmux status glyph expectations'
      - 'd7783ce2 chore: bump CLI version to 0.25.11'
      - '297e4a20 fix: restore runtime status and context conformance'
    behavioral_retrospective:
    - The release run initially failed under restricted sandbox DNS and then correctly
      reran with escalated network access.
    - A stale no-container E2E expectation around tmux status glyphs caught the new
      label symbols; the test was updated and committed before the release was rerun.
    - The user corrected the earlier approach to historical context hygiene; this
      session transformed the context tree to full conformance rather than treating
      old filenames and schemas as tolerated exceptions.
    - No deferred entity creation remains from this wrapup; the handover is recorded
      as a processkit LogEntry in this turn.
---
