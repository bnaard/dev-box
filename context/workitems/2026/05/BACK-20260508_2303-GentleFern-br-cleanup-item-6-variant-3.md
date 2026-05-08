---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2303-GentleFern-br-cleanup-item-6-variant-3
  created: '2026-05-08T23:03:43+00:00'
  labels:
    track: cleanup-arch
    release: v0.25.7
    deferred_from: v0.25.6 / DEC-20260508_2240-WarmLark
    parent_epic: BR-CLEANUP-ARCH
spec:
  title: 'v0.25.7: BR-CLEANUP-ARCH item 6 — Variant 3 Migration emission for drifted-but-not-historical
    files'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Goal

    Close out the final piece of the BR-CLEANUP-ARCH epic (DEC-20260508_1515-SilentAsh, BACK-20260508_1516-BrightStream). Items 1-5 shipped in commit `e0ee7bc` and the BrightStream session; item 6 was the only remaining piece.

    ## Spec (recovered from DEC-20260508_1515-SilentAsh)

    > "VARIANT 3 — MIGRATION NOTE ONLY (derived project's agent + user decide):
    > Anything aibox could classify as drifted but possibly intentional user customization — apply emits a pending Migration with per-file recommendation; the derived project's agent surfaces it on /pk-resume and /pk-doctor and walks it with the user via migration-management."

    From BACK-20260508_1516-BrightStream item 6:
    > "For files in the sync perimeter that match neither archived versions nor current generation, emit a Migration entity with a per-file recommendation."

    ## Implementation plan

    ### Detection branch (`cli/src/runtime_sync.rs`)
    Items 1-2 added the historical-managed-file recognizer. Item 6 needs the inverse:
    - For each file in the sync perimeter: (a) does its hash match canonical-current-generation? (b) does it match any archived-historical hash from the recognizer table?
    - If NEITHER → classify as **Variant 3** (drifted-but-possibly-intentional).
    - Collect the Variant 3 set across the apply pass.

    ### Migration entity emission (end of apply)
    - If Variant 3 set is non-empty, write a `MIG-RUNTIME-<timestamp>` Migration entity in the derived project's `context/migrations/pending/`.
    - Frontmatter shape mirrors the existing `MIG-RUNTIME-…` format.
    - Body: per-file table with columns `file | reason-for-classification | recommendation`. Recommendation values: `preserve-as-is | overwrite-with-canonical | review-manually`. Default `review-manually` until the recognizer can produce a justified recommendation.

    ### Surfacing
    Already wired in v0.25.6: `/pk-resume` lists pending migrations; `/pk-doctor migrations` check WARNs on stale-pending. No additional surfacing work.

    ### Acceptance
    - Integration test: create a drifted-but-not-historical file in `.aibox-home/.config/tmux/tmux.conf`, run `aibox apply`, assert (a) the file is left untouched, (b) a `MIG-RUNTIME-…` pending migration appears with the file listed and a recommendation, (c) `/pk-resume` surfaces the migration count, (d) `/pk-doctor` lists the migration.
    - All existing tests still green.

    ## Estimated effort
    ~1-2 hours for a senior engineer:
    - 30-45 min — detection branch + unit tests in `runtime_sync.rs`
    - 30-45 min — Migration entity body authoring + emit code path
    - 15-30 min — integration test

    ## Why deferred from v0.25.6

    Items 1-5 already shipped the architectural foundation. Variant 3 is the polish phase — improves UX (users see drift in a Migration instead of silent skip/overwrite) but does not fix a regression. Drifted-not-historical files currently get a "leave as-is" skip, which is functionally safe; this enhancement makes the drift visible to the agent + user via the Migration system. v0.25.6 already absorbed substantial scope expansion (v1-legacy cleanup, S3, Q7); a touch-the-runtime-sync-core change before tag was judged too late-stage to ship safely.

    ## Dispatch hint
    Avery (senior eng) — touches the runtime-cleanup architecture; primary surface is `cli/src/runtime_sync.rs`. Should fit in a single focused session.
---
