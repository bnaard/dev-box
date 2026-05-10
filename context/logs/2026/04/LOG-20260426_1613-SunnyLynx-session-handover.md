---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260426_1613-SunnyLynx-session-handover
  created: '2026-04-26T16:13:00+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-04-26T16:13:00+00:00'
  summary: Session handover — shipped aibox v0.21.1 (processkit v0.23.0 integration)
    and fixed gh-pages docs-deploy credential bug
  actor: claude-opus-4-7
  details:
    session_date: '2026-04-26'
    current_state: 'Tree clean on main @ e773442. aibox v0.21.1 fully shipped — Phase
      1 (Linux binaries + GH release + docs-deploy) and Phase 2 (macOS binaries +
      GHCR images on host) both complete. The release integrates processkit v0.23.0:
      50 mirror updates and 9 new upstream files (RoyalFern governance fields on 34
      model entries, new release-audit skill, pk-doctor skill_dag check, skill-finder
      catalog() tool, frontmatter.py block-scalar dumper, event-log actor-field validation).
      All 654 unit tests + 50 E2E + 16 integration pass. cargo clippy and cargo audit
      clean. docs-site live at https://projectious-work.github.io/aibox/. As a small
      follow-on, scripts/maintain.sh docs-deploy now injects the gh CLI OAuth token
      into the gh-pages push URL when the remote is HTTPS — the v0.21.1 release surfaced
      this credential failure inside the container.'
    open_threads:
    - CLI-11 Codex allowed_tools parity — deferred from v0.20.0, blocked on Codex
      shipping a permissions surface upstream
    - Upstream processkit PROVENANCE.toml stamping bug — v0.23.0 still ships generated_for_tag=v0.21.0;
      non-blocking warn-only, worth filing upstream when there's bandwidth
    - 'content_diff classification surfaced 50 ''conflicts'' for files that are actually
      upstream-only changes (live SHA == old-mirror SHA, but classifier labelled them
      conflict). aibox sync still wrote the right content, so no data was lost. Worth
      investigating: the v0.21.0 fix tightened RemovedUpstreamStale rules but left
      the regular conflict classifier with the same potential false-positive shape'
    - 'Issue #51 (OpenCode plugin research) — watch-and-act on upstream sst/opencode#2319'
    next_recommended_action: At session start, check whether processkit has shipped
      a release newer than v0.23.0 via `gh api repos/projectious-work/processkit/releases/latest
      --jq .tag_name`. If newer, begin the integration → patch-release cycle (./scripts/maintain.sh
      sync-processkit, then aibox sync, then maintain.sh release X.Y.Z). If still
      on v0.23.0, surface that there is no scheduled aibox work rather than padding
      with low-priority items, and consider opening an investigation WorkItem for
      the content_diff conflict-classification false-positive.
    branch: main
    commit: e773442
    behavioral_retrospective:
    - Migration plan for v0.22.0→v0.23.0 reported 50 'conflicts' but live==old-mirror
      on every one — the classifier is over-eager. Did not file a WorkItem mid-session;
      encoded as an open thread above so next session can decide whether to investigate.
    - docs-deploy gh-pages push has been failing silently in earlier release cycles
      too (the previous v0.21.0 cycle skipped docs-deploy in handover notes). Patched
      at root rather than working around — fix lives in scripts/maintain.sh and is
      verified end-to-end.
---
