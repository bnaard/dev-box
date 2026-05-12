---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_0004-DeepStone-session-handover
  created: '2026-05-12T00:04:53+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-12T00:04:39Z'
  summary: Session handover — aibox v0.25.9 released with processkit v0.26.1 and host-side
    completion verified
  actor: TEAMMEMBER-avery
  subject: aibox v0.25.9 release
  subject_kind: Release
  details:
    session_date: '2026-05-12'
    current_state: 'aibox v0.25.9 is released. The release includes processkit v0.26.1,
      applied processkit/runtime migrations, list-based tmux status work, Yazi OMP
      removal, runtime fixes, and the generated runtime refresh from host Phase 2.
      Linux-side and host-side release steps are complete: GitHub release assets for
      Linux and macOS are uploaded, GHCR images were pushed by the host step, runtime
      smoke passed, docs were deployed, and origin/main is clean at b993e50dcc08eb79c789715179ccaf8cdfb7da33.'
    open_threads:
    - No WorkItems are currently indexed as in_progress or blocked. The indexed WorkItem
      states show 5 items in review, including v0.25.6 CI/code-quality followups,
      security-hardening followups, stale-process/v1 cleanup, tmux runtime redesign,
      and tmux visual E2E rewrite.
    - The processkit MCP query_workitems/query_entities calls returned an Unexpected
      response type during wrapup; the same index data was read from context/.cache/processkit/index.sqlite
      as a fallback. Next session should consider diagnosing the gateway response
      shape if this repeats.
    - Host-side release reported runtime smoke passed with logs in dist/release-smoke/v0.25.9/.
      Those logs were not re-opened during wrapup because the user already reported
      host completion and GitHub/local verification passed.
    next_recommended_action: Run a short post-release verification from a derived
      project against v0.25.9, especially `aibox apply --standardize-config` and the
      tmux/Yazi status surfaces that were changed, then close or update the five WorkItems
      currently in review if the release evidence is sufficient.
    branch: main
    commit: b993e50
    working_tree: clean; main and origin/main both at b993e50dcc08eb79c789715179ccaf8cdfb7da33
    stash: none
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.25.9
    behavioral_retrospective:
    - No deferred entity creation was left outstanding. The release was committed,
      pushed, tagged, and verified before wrapup.
    - 'A tool-quality gap appeared: WorkItem query MCP calls failed with Unexpected
      response type. This was captured as an open thread rather than silently ignored.'
    - The final host-generated runtime refresh commit initially remained local; it
      was detected and pushed to origin/main before handover.
---
