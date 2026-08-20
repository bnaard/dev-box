---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260820_1712-SleekSwan-session-handover
  created: '2026-08-20T17:12:03+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-08-20T17:12:03+00:00'
  summary: Session handover — v0.34.1 fully released across Linux, macOS, GHCR, and
    documentation
  actor: codex
  subject: aibox-v0.34.1
  subject_kind: release
  details:
    session_date: '2026-08-20'
    current_state: 'aibox v0.34.1 is fully published. PRs #431, #432, and #433 are
      merged; the tag resolves to e4a2ab056547 on v0.x-release; Linux and Darwin release
      assets, checksums, LICENSE, documentation, and GHCR foundation/runtime images
      are live. The TeX Live addon now uses the reachable immutable TU Chemnitz 2025
      archive, and all release, visual, audit, doctor, and integrity gates passed.
      The repository has one clean worktree, no stashes, no open PRs, and only maintained
      branches.'
    open_threads:
    - BACK-20260820_1519-ShinyHeron-refresh-deferred-addon-pins remains in backlog
      for the non-security addon pin updates intentionally excluded from v0.34.1.
    - No WorkItems are currently in-progress or blocked.
    next_recommended_action: 'Start with BACK-20260820_1519-ShinyHeron-refresh-deferred-addon-pins:
      review upstream changes and checksums, then update pins with the affected image/addon
      regression gates.'
    branch: v0.x-release
    commit: e4a2ab056547
    behavioral_retrospective:
    - The release-only Starship gate exposed stale Projectious palette constants;
      the expectations were corrected and revalidated before publication.
    - The owner surfaced a derived-project TeX Live archive failure during release;
      the source addon mirror, release-state evidence, regression test, and release
      notes were fixed before restarting publication.
    - 'No promised tracking was left deferred: unrelated pin drift is recorded in
      BACK-20260820_1519-ShinyHeron-refresh-deferred-addon-pins.'
---
