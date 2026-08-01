---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260731_1854-SunnyOrchard-session-handover
  created: '2026-07-31T18:54:50+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-07-31T18:54:50+00:00'
  summary: 'Session handover — v0.28.19 released and host steps complete; v1 work
    preserved; issue #299 and companion repair remain'
  actor: TEAMMEMBER-avery
  subject: aibox
  subject_kind: Project
  details:
    session_date: '2026-07-31'
    current_state: 'aibox v0.28.19 is released, including the prerelease GHCR tag-generation
      fix that preserves full SemVer values such as 1.0.0-alpha.1. GitHub release
      assets, GHCR runtime/foundation images, documentation, protected-branch integration,
      and the user-run host Phase 2 were verified complete. GitHub Discussion #186
      was reviewed, answered, and closed; no open discussions remain. The checkout
      is clean on v0.x-release at 5cb830b9, while the pre-release v1.x workspace changes
      are preserved in stash@{0}.'
    open_threads:
    - 'GitHub issue #299 remains open: v1 stable requires real native rollback rehearsals
      on four platforms and external human operator pilots; it cannot be truthfully
      closed without that evidence.'
    - 'BACK-20260728_1004-GracefulSea-rebuild-companion-produce-live-m7c-evidence
      is in progress: rebuild/repair the aibox-e2e-testrunner companion and produce
      candidate-bound live M7c evidence.'
    - BACK-20260725_1003-GiftedBlossom-implement-m7c-disposable-cluster-e2e-and is
      blocked pending the disposable-cluster E2E/recovery evidence.
    - BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration remains in progress.
    - BACK-20260723_1538-MindfulAnchor-fix-kubernetes-addon-checksums-v0286 and BACK-20260721_1910-HardyClover-parallel-v0-v1-release-branches
      appear stale and should be reconciled against already-shipped work.
    - BACK-20260514_0925-VastHare-tmux-theme-switch-prefix-menu-binding and BACK-20260514_0924-ActiveSummit-tmux-layout-chooser-prefix-menu-binding
      remain in progress.
    - 'Tier 2 release validation was degraded: both parallel and serial runs wedged
      in the rootless Podman/Buildah addon build after companion access was restored.
      Local audit, format, clippy, unit, Tier-1, integration, rendered-color, and
      Linux cross-build gates passed.'
    next_recommended_action: Resume the preserved v1.x-pre-release workspace from
      stash@{0}, then repair/rebuild the E2E companion and complete BACK-20260728_1004-GracefulSea-rebuild-companion-produce-live-m7c-evidence
      before relying on Tier 2 or advancing v1 stable readiness.
    branch: v0.x-release
    commit: 5cb830b9
    worktree: clean
    stashes:
    - 'stash@{0}: On v1.x-pre-release: pre-release-host-v0.28.19-primary-20260731T184419Z'
    - 'stash@{1}: On docs/brand-v1-versioning-v1x: pre-release-host-v0.28.18-primary-20260730T203342Z'
    - 'stash@{2}: On v0.x-release: pre-release-host-v0.28.14-primary-20260726T164837Z'
    - 'stash@{3}: On v0.x-release: pre-release-host-v0.28.13-local-work-2026-07-26'
    - 'stash@{4}: On main: keep-aibox-toml-comment-drift'
    - 'stash@{5}: WIP on main: 29d22a12 fix(test): use family theme names in visual
      + visual_matrix'
    - 'stash@{6}: On main: pre-v0.25.14-release-unrelated-dirty-state'
    behavioral_retrospective:
    - 'No deferred user-facing commitments remain: the release, host verification,
      issue review, and discussion closure were completed.'
    - The isolated release worktree initially lacked the companion SSH-key projection
      and held a stale local release ref; both were diagnosed and corrected during
      the session.
    - The remaining companion degradation is already captured by BACK-20260728_1004-GracefulSea-rebuild-companion-produce-live-m7c-evidence,
      so no duplicate WorkItem was created.
---
