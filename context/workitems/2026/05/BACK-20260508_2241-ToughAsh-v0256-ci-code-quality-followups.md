---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2241-ToughAsh-v0256-ci-code-quality-followups
  created: '2026-05-08T22:41:02+00:00'
  labels:
    track: ci-code-quality
    release: v0.25.6
    deferred_via: DEC-20260508_2235-CuriousBadger
  updated: '2026-05-08T23:11:41+00:00'
spec:
  title: 'v0.25.6: CI + code-quality followups — Q5 release-smoke diffs, Q7 skills
    comment fact-check, BR-CLEANUP-ARCH item 6'
  state: review
  type: task
  priority: medium
  description: |
    ## Goal

    Resolve the three CI + code-quality items from the v0.25.6 deferred list before tagging.

    ## Items

    ### Q5 — Surface release-smoke status diffs in CI — DROPPED architecturally (2026-05-09)
    - Investigation 2026-05-09: `/workspace/.github/` contains only `repository-metadata.md`; there is no CI/CD pipeline at all. Owner clarified: "I have not paid github account and CI costs money for actions. We continue to do smoke tests during release with our scripts and our e2e companion container."
    - Recorded as `DEC-20260508_2257-SmoothFalcon-no-github-actions-ci-for-aibox`.
    - Q5 is not deferred — it is not on the project's roadmap. Release verification stays in `scripts/release-runtime-smoke.sh`, `scripts/aibox-upgrade-test.sh`, `scripts/maintain.sh test-e2e*`, all run from the developer's host as part of `./scripts/maintain.sh release` Phase 1.
    - **Status: DROPPED architecturally, not deferred. Future "add CI" proposals must address the cost gate first per DEC-20260508_2257.**

    ### Q7 — Streamline / fact-check skills `[skills]` inline comments in aibox.toml — KEEP (target for v0.25.6)
    - File: root `aibox.toml`.
    - Spec from LuckyLily Q7: walk the comment list once and fix any descriptions that are stale or contradicted by the current SKILL.md.
    - Mechanical pass; one general-purpose subagent (Robin) reads each `# processkit; <description>` line, reads the matching SKILL.md, and updates if drifted.

    ### BR-CLEANUP-ARCH item 6 — Variant 3 Migration emission — DEFERRED to v0.25.7 (2026-05-09)
    Owner accepted the scope+plan recommendation: defer to v0.25.7. Tracked at `BACK-20260508_2300-…-br-cleanup-arch-item-6-variant-3-migration-emission` (created in same turn as this update). Spec recovery + implementation plan + acceptance criteria preserved in the v0.25.7 WorkItem.

    Original spec presentation (kept for context):

    Spec recovered from DEC-20260508_1515-SilentAsh + BACK-20260508_1516-BrightStream:

    > "VARIANT 3 — MIGRATION NOTE ONLY (derived project's agent + user decide):
    > Anything aibox could classify as drifted but possibly intentional user customization — apply emits a pending Migration with per-file recommendation; the derived project's agent surfaces it on /pk-resume and /pk-doctor and walks it with the user via migration-management."
    > Item 6 (BrightStream): "For files in the sync perimeter that match neither archived versions nor current generation, emit a Migration entity with a per-file recommendation."

    Items 1-5 shipped in commit e0ee7bc + the BrightStream session (2026-05-08). Item 6 was the only deferred piece.

    **Implementation surface:**
    - File: `cli/src/runtime_sync.rs` (sync perimeter logic; already has historical-managed-file recognizer from items 1-2).
    - Add a "drifted-but-not-historical" detection branch: when a file in the sync perimeter matches neither (a) the canonical-current-generation hash nor (b) any archived-historical hash from the recognizer table, classify it as Variant 3.
    - Collect the Variant 3 set across the apply pass.
    - At end of apply, if Variant 3 set is non-empty, emit a `MIG-RUNTIME-<timestamp>` Migration entity in the derived project's `context/migrations/pending/` listing each drifted file with a per-file recommendation field (`preserve-as-is | overwrite-with-canonical | review-manually`).
    - Surfacing is already wired: `/pk-resume` and `/pk-doctor migrations` both list pending migrations.

    **Estimated scope:** ~1-2 hours for Avery (senior eng).
    - 30-45 min: detection branch + tests in `runtime_sync.rs`.
    - 30-45 min: Migration entity body authoring (frontmatter + per-file table).
    - 15-30 min: integration test that creates a drifted file, runs apply, asserts the pending migration appears with the right recommendation.

    **Risk:** medium. Touches `runtime_sync.rs` core. Mitigated by the e2e tests already in place for items 1-5 (BrightStream's acceptance criteria added e2e coverage).

    **Recommendation: defer to v0.25.7.** Items 1-5 already shipped the architectural foundation; Variant 3 is the polish phase. It improves UX (users see drift instead of silent overwrite/skip) but does not fix a regression. Punting does not break apply correctness — drifted-not-historical files just continue their current treatment (no-op skip, equivalent to historical "leave as-is"). v0.25.6 has already absorbed scope expansion this session.

    Awaiting owner call on v0.25.6 vs v0.25.7.

    ## Dispatch hint
    - Q5: Avery (senior eng) — needs CI workflow knowledge and judgement on the Docker/non-Docker trade-off.
    - Q7: Robin (junior eng / mechanical) — large but mechanical pass.
    - BR item 6: Avery — touches the runtime-cleanup architecture; recovery of the exact spec is the first step.

    ## Acceptance
    - All three either land or get re-deferred via fresh DEC.
    - Release notes mention Q5 (CI gate) and BR-CLEANUP item 6 if shipped.
  started_at: '2026-05-08T23:03:05+00:00'
---

## Transition note (2026-05-08T23:03:05+00:00)

Starting Q7 (skills inline-comment fact-check pass) via Robin. Q5 dropped per DEC-20260508_2257-SmoothFalcon (no GitHub Actions / CI cost gate). BR-6 deferred to v0.25.7.


## Transition note (2026-05-08T23:11:41+00:00)

All three items resolved.

Q7 (skills inline-comment fact-check): NO-OP. Robin walked all 140 lines in the [skills] block (44 enabled processkit + 96 disabled non-processkit). Compared each `# <category>; <description>` against the `description:` field of the matching SKILL.md. Result: **0 drifted**. Every inline comment is either an exact match or a faithful prefix-truncation of the SKILL.md description. aibox.toml unchanged (555 lines before/after).

Notable side-finding from Q7 (not in scope, flagged for awareness): the 96 disabled non-processkit skills exist only in `context/templates/processkit/v0.25.6/` upstream mirror — they are not in the active `context/skills/` catalog. Working as designed (they get copied on enable + apply), but if Q7 were extended to "verify install-state consistency" it would warrant attention. Out of scope for v0.25.6.

Q5 (release-smoke status diffs in CI): DROPPED architecturally per DEC-20260508_2257-SmoothFalcon-no-github-actions-ci-for-aibox (cost gate; project does not adopt paid CI; release verification stays in scripts + e2e companion).

BR-CLEANUP-ARCH item 6 (Variant 3 Migration emission): DEFERRED to v0.25.7. Tracked at BACK-20260508_2303-GentleFern-br-cleanup-item-6-variant-3 with full spec recovery + implementation plan.

Ready for owner review.
