---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260508_1311-RoyalBrook-session-handover
  created: '2026-05-08T13:11:04+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-08T13:10:41Z'
  summary: Session handover - v0.25.5 release completed end to end
  actor: codex
  details:
    session_date: '2026-05-08'
    current_state: aibox v0.25.5 is released end to end. Linux-side release completed
      from the container, main was pushed, tag v0.25.5 was created, GitHub release
      assets were uploaded, docs were deployed, and the user confirmed host-side release
      steps completed. Post-confirmation verification showed the release has all four
      binary assets, runtime smoke artifacts exist under dist/release-smoke/v0.25.5/20260508-150814/,
      and local main is clean and aligned with origin/main at cdd2f37.
    open_threads:
    - No WorkItems are currently in_progress or blocked.
    - BACK-20260508_1214-SureSeal-review-uv-image-update-after-v0255 remains in backlog
      to evaluate ghcr.io/astral-sh/uv 0.11.10 -> 0.11.11 after the patch release.
    - BACK-20260508_1214-TallFrog-review-wasm-bindgen-updates remains in backlog to
      review js-sys and wasm-bindgen lockfile updates after the patch release.
    next_recommended_action: Start the next session by running pk-resume and then
      decide whether to pick up the two v0.25.5 dependency follow-up WorkItems, beginning
      with the uv image update because it affects generated runtime/container image
      behavior.
    branch: main
    commit: cdd2f37
    release: v0.25.5
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.25.5
    validation:
    - cargo fmt -- --check passed in release script
    - cargo clippy --all-targets -- -D warnings passed in release script
    - cargo test passed in release script
    - 'Tier 2 SSH companion E2E passed: 109 passed, 3 release-gated visual tests ignored
      in base tier'
    - AIBOX_RELEASE_VISUAL_E2E=full passed all three opt-in visual tiers
    - cargo audit passed
    - Linux aarch64 and x86_64 artifacts built and uploaded
    - macOS aarch64 and x86_64 artifacts verified after host-side completion
    - docs deployed to GitHub Pages
    - runtime smoke passed on host with logs under dist/release-smoke/v0.25.5/20260508-150814/
    git_context:
      branch: main
      commit: cdd2f37
      status: clean and aligned with origin/main
      stash: none
      recent_commits:
      - 'cdd2f37 chore: refresh generated runtime for v0.25.5'
      - 'd1dbae9 test: harden visual keybinding e2e timing'
      - 'f27bd56 test: align visual matrix with managed tmux socket'
      - '39d22a8 fix: repair companion runtime editor handoff'
      - '95f3131 chore: record v0.25.5 dependency deferrals'
      - '25a0945 chore: bump CLI version to 0.25.5'
      - '86b8cc4 fix: refresh tmux runtime before session recreate'
    behavioral_retrospective:
    - The release exposed that the E2E companion image was missing durable Podman/rootless
      networking tools; this was fixed in .devcontainer/Dockerfile.e2e and validated
      by full companion E2E.
    - The tmux managed socket migration required aligning visual matrix tests with
      the managed socket; this was fixed rather than weakening coverage.
    - The vim-loop keep-alive behavior changed visual keybinding assumptions; tests
      were hardened to wait for real Yazi render state and inspect target-pane command
      state instead of global process state.
    - 'No deferred promise from this wrapup remains unrecorded: dependency follow-ups
      already have backlog WorkItems and this handover captures the release state.'
---
