---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1519-PluckyThorn-release-host-orchestration-rollout-cutover
  created: '2026-05-08T15:19:43+00:00'
  labels:
    track: release-rollout
    release: v0.25.6
  updated: '2026-05-08T21:47:24+00:00'
spec:
  title: 'v0.25.6: Release-host orchestration and rollout'
  state: review
  type: task
  priority: medium
  description: |
    ## Goal
    Cut and roll out v0.25.6 once the five implementation tracks have landed (BR-CLEANUP-ARCH, BR-ZELLIJ-EXCISE, BR-DOCTOR-GAPS, BR-TEST-GAPS, BR-SEC-HARDEN, BR-CODE-QUALITY).

    ## Steps

    ### R1 — Lockfile schema migration document
    - Generate Migration entity in context/migrations/pending/ covering the lock-schema bump from BR-CLEANUP-ARCH item 1 (`previous_selection` backfill).

    ### R2 — Release notes
    - File: cli/src/compat.rs (new entry in the RELEASES table) + docs-site changelog if present.
    - Items: cross-version sync auto-recovers corrupted managed runtime files; generic purge-on-disable for all addon tools; new [apply].purge_disabled_harness_state; zellij scorched-earth excision; new doctor checks; addon download integrity hardening; aibox.toml [skills] dedup; seccomp=unconfined now requires explicit consent.

    ### R3 — release-runtime-smoke baseline refresh
    - After all tracks land, run scripts/release-runtime-smoke.sh for v0.25.6 and commit dist/release-smoke/v0.25.6/.

    ### R4 — Migration message for derived projects
    - v0.25.6 introduces auto-recovery of managed runtime files; files that look corrupted will be regenerated. Opt-out by placing outside sync perimeter.
    - v0.25.6 hard-purges any zellij artifact. Breaking change — see docs/migrations/zellij-eol.md.

    ### R5 — Tag, build, publish
    - Standard release-audit + tag + build via scripts/release-check-state.sh and scripts/release-runtime-smoke.sh.

    ## Acceptance criteria
    - v0.25.6 tagged on main with all five implementation tracks merged.
    - dist/release-smoke/v0.25.6/ baseline committed.
    - Release notes call out every breaking change with remediation.

    ## Dispatch hint for next session
    Wrap-up workitem; do it last. Coordinator agent handles directly with one or two short follow-up agents for documentation polish.
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
---

## Transition note (2026-05-08T21:41:58+00:00)

All 9 blockers shipped this session. Splitting work: R1 Migration entity via MCP (Cora), R2+R4 release notes + EOL doc (Jordan), R3 deferred (Docker infra), R5 stops for human authorisation.


## Transition note (2026-05-08T21:47:24+00:00)

R1 (lockfile migration entity), R2 (release notes in compat.rs::COMPAT_TABLE), R4 (zellij-eol.md + lockfile-v0-25-6.md migration docs) shipped. R3 (release-runtime-smoke baseline refresh, requires Docker) and R5 (tag/build/publish — irreversible, requires human authorisation) intentionally deferred. Reviewer should treat this as "ready for human cutover" rather than "all sub-tasks complete".
