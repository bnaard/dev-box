---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260508_1147-KeenMoss-session-handover
  created: '2026-05-08T11:47:17+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-08T11:47:01Z'
  summary: Session handover - aibox v0.25.4 released end-to-end and host Phase 2 completed
  actor: Codex
  subject: aibox v0.25.4 release wrapup
  subject_kind: Release
  details:
    session_date: '2026-05-08'
    current_state: aibox v0.25.4 is released end-to-end. Linux Phase 1 produced and uploaded the Linux assets, deployed docs, and pushed tag v0.25.4; host Phase 2 uploaded macOS assets, pushed GHCR images, refreshed generated runtime surfaces, committed them, and passed runtime smoke. A follow-up cleanup removed the redundant host-side docs deployment note from scripts/maintain.sh and pushed main to 313d369. The working tree is clean at handover.
    open_threads:
    - v0.25.3 remains published but was superseded by v0.25.4 after host runtime smoke exposed the tmux socket probe and status-right rendering defects.
    - Companion SSH E2E was intentionally excluded from the v0.25.4 release via AIBOX_RELEASE_SKIP_COMPANION_E2E=1; run the companion E2E separately when the companion environment should be revalidated.
    - Visual E2E was intentionally skipped via AIBOX_RELEASE_VISUAL_E2E=skip; the standard unit, no-container E2E, integration, clippy, audit, Linux build, host smoke, macOS asset, and GHCR gates completed.
    next_recommended_action: Start the next session by running pk-resume and checking whether the owner wants a post-release issue/retro for v0.25.3 being superseded by v0.25.4, or whether to proceed with the next backlog item now that the release is complete.
    branch: main
    commit: 313d369
    stash: none
    release:
      tag: v0.25.4
      url: https://github.com/projectious-work/aibox/releases/tag/v0.25.4
      assets:
      - aibox-v0.25.4-aarch64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.4-x86_64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.4-aarch64-apple-darwin.tar.gz
      - aibox-v0.25.4-x86_64-apple-darwin.tar.gz
      host_runtime_smoke: passed
      host_runtime_smoke_logs: dist/release-smoke/v0.25.4/20260508-133840/
      docs: deployed during Phase 1
    completed_work:
    - 'Committed and pushed chore: remove redundant host docs reminder (313d369).'
    - Published v0.25.4 as the replacement patch release after repairing tmux release-host smoke socket probing and status-right placeholder replacement.
    - Verified GitHub release v0.25.4 has Linux and macOS assets uploaded and is not draft/prerelease.
    behavioral_retrospective:
    - The user had to confirm host Phase 2 after v0.25.4; that is expected because Phase 2 runs on macOS host, but the handover now records the host evidence explicitly.
    - The stale docs deployment note in release-host output was removed so future host completion summaries do not imply docs are still pending after Phase 1.
---
