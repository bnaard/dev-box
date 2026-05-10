---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0843-WiseClover-phase-0-of-release-ritual-should
  created: '2026-05-10T08:43:10+00:00'
  labels:
    version: v0.25.7-followup
    area: release-process
    github_issue: '73'
  updated: '2026-05-10T09:47:03+00:00'
spec:
  title: Phase 0 of release ritual should run pk-doctor + aibox doctor before bump-version
    (gh#73)
  state: done
  type: task
  priority: high
  description: |
    ## Source

    GitHub issue: https://github.com/projectious-work/aibox/issues/73

    ## Problem

    The canonical aibox release ritual at `context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` defines a Phase 0 ("Dependency and harness state check, Claude does this FIRST") that runs `./scripts/maintain.sh release-check-state` to write `dist/RELEASE-STATE.md`. It does NOT run `pk-doctor` or `aibox doctor` — both of which exist and are designed exactly for "is this repo healthy enough to release?"

    Concrete failure mode: aibox @ v0.25.6 release shipped re-planning loops because v1-stale entities misled `route_task`. A pre-flight `pk-doctor` would have surfaced the drift.

    ## Proposed fix (per the issue)

    Wire both doctors into Phase 0 of `NOTE-20260411`:

    ```bash
    ./scripts/maintain.sh release-check-state    # existing
    pk-doctor                                    # new — processkit health
    aibox doctor                                 # new — aibox runtime hygiene
    ```

    **Gate semantics** (per the issue's recommendation):
    - ERRORs from either doctor block the release.
    - WARNs surface to the release notes / handover but don't block.
    - `scripts/maintain.sh release` invokes both doctors after release-check-state, before bump-version.

    ## Files likely touched

    - `scripts/maintain.sh` — `cmd_release_check_state` and/or `cmd_release` to invoke the doctors and gate on ERROR exit codes.
    - `context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` — Phase 0 section.
    - `AGENTS.md` — "ship it" / release section.

    ## Acceptance

    - `scripts/maintain.sh release-check-state` (or the next phase) calls both doctors.
    - ERROR exit from either doctor halts release with a clear message; WARN doesn't block.
    - The release-process note documents the new sequence.
    - Manual smoke-test: run the release pipeline locally up to the doctor phase; verify both invoked, verify ERROR gate works.

    ## PR commit message convention

    Reference `Closes #73` or `Fixes #73` in the commit body so GitHub auto-closes on merge.
  started_at: '2026-05-10T09:46:29+00:00'
  completed_at: '2026-05-10T09:47:03+00:00'
---

## Transition note (2026-05-10T09:47:03+00:00)

Implemented and pushed in commit c8fe490 (closed gh#73). scripts/maintain.sh::cmd_release_doctors invokes pk-doctor (uv run script form) + aibox doctor; combined output → dist/RELEASE-DOCTORS.md; ERRORs block, WARNs surface. NOTE-LoyalSpruce + AGENTS.md updated. GitHub issue closed manually (auto-close didn't fire on direct push but the commit body has Closes #73).
