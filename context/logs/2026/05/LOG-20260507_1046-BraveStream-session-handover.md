---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_1046-BraveStream-session-handover
  created: '2026-05-07T10:46:25+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T10:46:25+00:00'
  summary: "Session handover \u2014 aibox v0.24.1 release completed end-to-end"
  actor: codex
  subject: aibox v0.24.1
  subject_kind: Release
  details:
    session_date: '2026-05-07'
    current_state: aibox v0.24.1 is released end-to-end. The release repaired the v0.24.0 host-smoke regression by removing the generated Compose main-service user override so the base image can start its root entrypoint, perform UID/GID and ownership setup, then drop to aibox as designed. The v0.24.1 GitHub release is public with Linux and macOS binaries for both supported architectures; the user completed host-side Phase 2, pushed GHCR images, passed runtime smoke, and refreshed generated runtime. The repo is clean and synced with origin/main at 0a09c87, with tag v0.24.1 still pointing at the release-script version bump commit 863ea72 and post-tag generated-runtime/bookkeeping commits on main.
    open_threads:
    - No WorkItems are currently in progress or blocked according to processkit query_entities.
    - 'One old stash remains: stash@{0}: On main: wip: interrupted v0.23.19 generated-runtime state. It predates this release work and was not touched.'
    - Companion/visual E2E were skipped during the Linux-side release with explicit release skip knobs because this active dev-container does not have the companion service available. Host runtime smoke for v0.24.1 passed, covering the release blocker that v0.24.0 exposed.
    - The host prompt notes docs-deploy can be run from the dev-container if needed. Docs were already deployed during the Linux-side maintain.sh release 0.24.1 run.
    next_recommended_action: Start the next session with pk-resume, confirm the old v0.23.19 stash is still intentionally retained or safe to delete, then proceed with normal post-release development. If validating release artifacts again, first check v0.24.1 GHCR tags and the smoke logs under dist/release-smoke/v0.24.1/.
    branch: main
    commit: 0a09c87
    tag: v0.24.1
    tag_commit: 863ea72
    post_tag_commits:
    - 'd378eaa chore: refresh generated runtime for v0.24.1'
    - '0a09c87 chore: record v0.24.1 release completion'
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.24.1
    release_assets:
    - aibox-v0.24.1-aarch64-apple-darwin.tar.gz
    - aibox-v0.24.1-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.24.1-x86_64-apple-darwin.tar.gz
    - aibox-v0.24.1-x86_64-unknown-linux-gnu.tar.gz
    git_status: clean; main synced with origin/main
    stash: 'stash@{0}: On main: wip: interrupted v0.23.19 generated-runtime state'
    validation:
    - cargo test passed during Linux-side release
    - cargo clippy --all-targets -- -D warnings passed during Linux-side release
    - cargo audit passed during Linux-side release
    - Linux release builds and aibox --version smoke passed during Linux-side release
    - GitHub release v0.24.1 verified with four uploaded CLI assets
    - 'User confirmed host-side release-host 0.24.1 completed: macOS binaries uploaded, GHCR images pushed, runtime smoke passed, generated runtime refreshed'
    behavioral_retrospective:
    - 'The central runtime lesson was that user: aibox in Compose was less safe for this image because it bypassed the privileged entrypoint contract; the durable fix is to let the image start with its default root entrypoint and drop privileges internally.'
    - The investigation benefited from host smoke artifacts and exact container logs. Future release-smoke failures should start from dist/release-smoke/<version>/ before speculating about runtime internals.
    - The processkit release completion event had to use the declared schema event type release.published; rejected ad hoc event types were not left as files.
    - No additional AGENTS.md or skill-rule change was made during wrapup because the relevant lessons are already encoded in the release process and in this handover.
---
