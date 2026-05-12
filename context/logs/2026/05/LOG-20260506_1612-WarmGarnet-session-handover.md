---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260506_1612-WarmGarnet-session-handover
  created: '2026-05-06T16:12:55+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-06T16:12:55+00:00'
  summary: 'Session handover: aibox v0.23.20 release recovery completed through host Phase 2.'
  actor: TEAMMEMBER-20260422_0832-MigratedMember-cora
  subject: aibox-v0.23.20-release-recovery
  subject_kind: Release
  details:
    handover_type: end_of_session
    date: '2026-05-06'
    repo: /workspace
    active_interlocutor: Cora (TEAMMEMBER-20260422_0832-MigratedMember-cora; ROLE-20260422_0001-MigratedRole-product-manager/senior)
    completed:
    - Recovered release stability by stepping through v0.23.16, v0.23.17, v0.23.18, and v0.23.19 with the host-safe runtime smoke harness.
    - Identified older release failures as overlapping Yazi 26 config/schema regressions plus an unsafe host smoke harness that streamed raw TUI escape sequences.
    - 'Committed and pushed the durable host-safe smoke harness fix in e62ad5e: default shell Zellij status mode for release smoke, capture raw TUI output to logs, and assert on structured probe markers instead of terminal transcripts.'
    - Recorded recovery baseline decision DEC-20260506_1243-WarmBadger-recover-aibox-release-stability-from-v0 and patch-release decision DEC-20260506_1546-GrandGlade-ship-patch-release-for-host-safe.
    - 'Published v0.23.20 Phase 1 from the container: version bump, compat/docs entry, tests, clippy, cargo audit, Linux release binaries, tag, GitHub release, and docs deploy.'
    - Fixed release-host repo inference bug in 89567f2 by making gh release operations use explicit projectious-work/aibox with AIBOX_GITHUB_REPO override.
    - 'User completed host Phase 2: macOS binaries uploaded, GHCR container images pushed, runtime smoke passed, and generated runtime surfaces refreshed and pushed in b4e9a15.'
    validation:
    - 'Final git status in /workspace: clean, main aligned with origin/main.'
    - Remote main verified at b4e9a159a2e2b0230c180f70b4a81b9250db85ed.
    - Tag v0.23.20 verified at ea9dae1f1ae5493b0ae15cb7afa95e49393065d1.
    - 'GitHub release v0.23.20 is public and non-prerelease with four assets: aarch64/x86_64 Linux and aarch64/x86_64 macOS.'
    - Release script validation included cargo fmt --check, clippy -D warnings, full cargo test, Tier 2 SSH companion E2E 113 passed / 3 ignored, cargo audit, Linux release builds, and version smoke.
    - GHCR package API verification from the container was blocked by missing read:packages scope, but host release-host reported image push complete and runtime smoke passed.
    current_state:
      main_head: 'b4e9a15 chore: refresh generated runtime for v0.23.20'
      recent_commits:
      - 'b4e9a15 chore: refresh generated runtime for v0.23.20'
      - '89567f2 fix: make release GitHub repo explicit'
      - 'ab3fcbe docs: add v0.23.20 compatibility entry'
      - '6b339d9 chore: bump CLI version to 0.23.20'
      - '92cf374 docs: record host-safe smoke patch release decision'
      - 'e62ad5e test: make release runtime smoke host-safe'
      - '41cef4e docs: record release recovery baseline decision'
      - 'b7c799c fix: run release smoke init non-interactively'
      github_release: https://github.com/projectious-work/aibox/releases/tag/v0.23.20
      workspace_status: clean
    open_threads:
    - The source tag v0.23.20 does not include commit 89567f2 because that release-host repo-explicit fix was pushed after the tag. It is on main for future releases.
    - GHCR direct API verification still requires a token with read:packages if we want machine-verified package metadata from inside the container.
    - The opt-in visual E2E matrix was skipped by release default; Tier 2 covered the relevant runtime/TUI surfaces, but a future release could choose AIBOX_RELEASE_VISUAL_E2E=status|tabs|yazi|full for broader screenshots.
    next_actions:
    - No immediate release action remains for v0.23.20.
    - For the next release, use the updated main branch so release-host resolves GitHub release operations against projectious-work/aibox explicitly.
    - If doing a release retrospective, focus on why host Phase 2 smoke was first exercised only after multiple TUI/runtime changes and add a policy for validating host-safe log capture before release tagging.
---
