---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260511_1804-QuietPine-session-handover
  created: '2026-05-11T18:04:02+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-11T18:03:40Z'
  summary: Session handover - v0.25.8 release completed end to end
  actor: Codex
  details:
    session_date: '2026-05-11'
    current_state: aibox v0.25.8 is complete end to end. Linux-side release, GitHub release assets, docs deployment, macOS host binaries, GHCR images, and host runtime smoke all passed. main is pushed to origin at 0c5504e, with the host-side generated runtime refresh committed by the user/host workflow after the release. The working tree has one pre-existing uncommitted change in aibox.toml, apparently a generated config/comment refresh around preview-tool comments and harness_order documentation; I did not modify or stage it during wrapup.
    open_threads:
    - No WorkItems are currently indexed as in_progress or blocked.
    - 'BACK-20260511_1528-StoutStream-review-uv-base-image-pin-update remains a backlog follow-up from release-state reporting: uv image pin 0.11.11 -> 0.11.13 should be reviewed separately.'
    - Uncommitted aibox.toml change remains in the working tree and should be explicitly reviewed, committed, or reverted by the next session/owner before new release work.
    - 'Potential release-hardening follow-up: make release doctors set AIBOX_ADDONS_DIR to the repo addons path and make pk-doctor use a writable UV_CACHE_DIR/fallback in constrained automation contexts.'
    next_recommended_action: Start the next session by inspecting the uncommitted aibox.toml diff, deciding whether it is an intended generated-config refresh, and either committing it or reverting it before starting unrelated work.
    branch: main
    commit: 0c5504e
    git_status: 'main is aligned with origin/main; uncommitted: aibox.toml; no stashes.'
    behavioral_retrospective:
    - The host Phase 2 smoke initially failed because scripts/release-runtime-smoke.sh still expected old tmux window names (ai/git). I fixed and pushed 7d798e0 to check the new layout names (work/lazygit/shell) and updated the stale MCP regex.
    - 'The first in-container release attempt hit sandbox-specific doctor issues: pk-doctor could not write to the uv cache and aibox doctor lacked AIBOX_ADDONS_DIR. The release succeeded after rerunning outside sandbox constraints with AIBOX_ADDONS_DIR=/workspace/addons; this should be encoded as release-script hardening rather than relying on agent memory.'
    - The v0.25.8 release required an additional compat table entry after the version bump. That was fixed in a616473 and the focused compat test passed before rerunning release.
---
