---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260508_0736-SoundEmber-session-handover
  created: '2026-05-08T07:36:15+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-08T07:36:15+00:00'
  summary: "Session handover \u2014 aibox v0.25.2 released end-to-end and host Phase 2 completed"
  actor: Codex
  subject: aibox v0.25.2 release wrapup
  subject_kind: release-session
  details:
    session_date: '2026-05-08'
    current_state: aibox v0.25.2 is released end-to-end. Phase 1 produced Linux assets and GitHub release v0.25.2; host Phase 2 then uploaded macOS assets, pushed GHCR images, and passed release runtime smoke. A follow-up commit 68a1d87 fixed the non-interactive release attach smoke path and recorded the v0.25.2 generated runtime refresh. The repository is clean on main and matches origin/main.
    open_threads:
    - BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign remains in-progress as the parent epic; several implementation children are in review.
    - 'Review/close the v0.25.2 tmux runtime review items: BACK-20260508_0435-SnappySwan-expose-provider-endpoint-url-variables-from, BACK-20260508_0425-SilentCrane-implement-amended-two-line-tmux-status, BACK-20260508_0356-QuickGarnet-implement-standard-tmux-powerkit-keybindings-popup, plus the earlier tmux migration review children.'
    - 'Release-check deferrals are tracked in backlog: BACK-20260508_0629-GentleSeal-defer-rust-crate-updates and BACK-20260508_0629-ToughTide-defer-uv-image-pin.'
    - 'Docs deployment was noted as a remaining optional inside-devcontainer step: ./scripts/maintain.sh docs-deploy.'
    - 'A stale stash still exists: stash@{0}: On main: wip: interrupted v0.23.19 generated-runtime state. It predates this session and was not touched.'
    - 'The initial host smoke failure was not the old ''can''t find pane: 1'' regression; the smoke reached the attach boundary and Docker rejected non-TTY stdin. scripts/release-runtime-smoke.sh now treats that boundary as success while still checking for the pane-index regression.'
    next_recommended_action: Start by reviewing and closing the v0.25.2/tmux runtime workitems currently in review, then decide whether to run docs deployment from inside the dev-container.
    branch: main
    commit: 68a1d87
    git_status: clean; main matches origin/main
    release:
      version: 0.25.2
      tag: v0.25.2
      url: https://github.com/projectious-work/aibox/releases/tag/v0.25.2
      assets_verified:
      - aibox-v0.25.2-aarch64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.2-x86_64-unknown-linux-gnu.tar.gz
      - aibox-v0.25.2-aarch64-apple-darwin.tar.gz
      - aibox-v0.25.2-x86_64-apple-darwin.tar.gz
      phase_2: 'complete: macOS binaries uploaded, GHCR images pushed, runtime smoke passed'
    behavioral_retrospective:
    - The first release delegation completed publication but skipped companion/visual E2E because the runtime lacked docker/podman; this was reported and release-check deferrals were tracked.
    - The host Phase 2 smoke exposed a script bug after release; it was fixed immediately and pushed as 68a1d87.
    - No deferred entity creation remains from this wrapup; release deferrals and decisions already exist in processkit.
---
