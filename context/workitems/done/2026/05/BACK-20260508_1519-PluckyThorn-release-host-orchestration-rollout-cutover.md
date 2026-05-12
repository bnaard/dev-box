---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1519-PluckyThorn-release-host-orchestration-rollout-cutover
  created: '2026-05-08T15:19:43+00:00'
  labels:
    track: release-rollout
    release: v0.25.6
  updated: '2026-05-09T05:11:41+00:00'
spec:
  title: 'v0.25.6: Release-host orchestration and rollout'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  blocked_by:
  - BACK-20260508_1516-BrightStream-stale-state-cleanup-architecture-foundational
  - BACK-20260508_1517-TrueBrook-zellij-scorched-earth-complete-excision
  - BACK-20260508_1517-SnowyWillow-doctor-coverage-gap-closure-work
  - BACK-20260508_1518-KeenBison-e2e-companion-test-gap-closure
  - BACK-20260508_1518-HonestAnt-addon-installer-security-hardening-checksum
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  - BACK-20260508_1603-QuietCedar-status-bar-visual-rework-powerline
  - BACK-20260508_1603-SilentEagle-log-pane-reader-lnav-integration
  - BACK-20260508_1604-GrandWillow-yazi-vim-pane-hard-cut
  started_at: '2026-05-08T21:41:58+00:00'
  completed_at: '2026-05-09T05:11:41+00:00'
---

## Transition note (2026-05-08T21:41:58+00:00)

All 9 blockers shipped this session. Splitting work: R1 Migration entity via MCP (Cora), R2+R4 release notes + EOL doc (Jordan), R3 deferred (Docker infra), R5 stops for human authorisation.


## Transition note (2026-05-08T21:47:24+00:00)

R1 (lockfile migration entity), R2 (release notes in compat.rs::COMPAT_TABLE), R4 (zellij-eol.md + lockfile-v0-25-6.md migration docs) shipped. R3 (release-runtime-smoke baseline refresh, requires Docker) and R5 (tag/build/publish — irreversible, requires human authorisation) intentionally deferred. Reviewer should treat this as "ready for human cutover" rather than "all sub-tasks complete".


## Transition note (2026-05-09T05:11:41+00:00)

v0.25.6 shipped 2026-05-09T05:01:34Z.

- Tag pushed: https://github.com/projectious-work/aibox/releases/tag/v0.25.6
- GitHub release "aibox v0.25.6" with 2 Linux binary assets:
  - aibox-v0.25.6-aarch64-unknown-linux-gnu.tar.gz (4.5 MB)
  - aibox-v0.25.6-x86_64-unknown-linux-gnu.tar.gz (4.7 MB)
- Docs deployed to https://projectious-work.github.io/aibox/ (gh-pages 4eaeb7f)

Phase 1 commits (this session):
- ff42d1e — v1-legacy processes/+actors/ archive + 5 GH issues filed
- 47afe60 — S3 Codex seccomp consent gate + remaining deferred items resolved
- 7ee5557 — LoyalSpruce zellij→tmux doc fix
- adafc78 — chore: bump CLI version to 0.25.6 (auto by maintain.sh release)
- ac0d486 — cargo fmt drift fix
- 35af498 — fix 2 e2e tests (visual_kb_yazi_e for popup arch; lazygit regex tighten)

Phase 2 remaining (owner's host):
- `./scripts/maintain.sh release-host 0.25.6` on macOS for darwin arm64 + x86_64 binaries and GHCR image push
- Lockfile backfill (`aibox apply` from a v0.25.6 host CLI to populate previous_selection fields)

v0.25.6 release ritual complete from this container. Ready for owner Phase 2.
