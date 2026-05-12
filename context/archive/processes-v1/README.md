# Archive — `context/processes/` (v1 legacy)

This directory was moved here on 2026-05-09 as part of the v0.25.6
stale-process / v1-legacy cleanup
(BACK-20260508_2241-QuietLark, DEC-20260508_2247-FierceQuail).

## Why archived

All 7 entities below carry `apiVersion: processkit.projectious.work/v1`,
the legacy schema for the v1 `Process` entity type. The v2 model
replaces v1 Process with Scope + Gate + process-instance composition.
The skill catalog explicitly marks `process-management` as
"Legacy/migration guidance for v1 Process entities".

In addition, none of the live aibox release ritual flowed through
`PROC-release.md` — the canonical release process for aibox lives in:

- `context/notes/2026/04/NOTE-20260411_0001-LoyalSpruce-aibox-release-process.md`
  (Phase 0 / Phase 1 / Phase 2 step-by-step)
- `AGENTS.md:139` (entry point)

`PROC-release.md`'s generic 6-step "update CHANGELOG, bump version,
tag, publish, announce" outline misled the v0.25.6 release agent twice
in one session (see DEC-20260508_2240-WarmLark and the post-release
retrospective). That alone was the trigger to archive.

## What's here

| File | Original purpose | Live successor |
|---|---|---|
| `INDEX.md` | Process directory index | n/a — v1 directory contract |
| `PROC-release.md` | Generic release process | `context/notes/2026/04/NOTE-20260411_0001-LoyalSpruce-aibox-release-process.md` + `AGENTS.md:139` |
| `PROC-bug-fix.md` | Generic bug-fix process | `AGENTS.md` engineering guidance |
| `PROC-code-review.md` | Generic code-review process | `AGENTS.md` PR / review section |
| `PROC-feature-development.md` | Generic feature workflow | `AGENTS.md` + workitem-management skill |
| `PROC-backlog-grooming.md` | Backlog-grooming workflow | context-grooming skill (`/pk-groom`) |
| `team-task-distribution.md` | Task distribution among Actors | team-manager skill + AGENTS.md role table |

The 4 slug-named duplicates that previously sat alongside these
(`release.md`, `bug-fix.md`, `code-review.md`, `feature-development.md`)
were `git rm`'d outright on 2026-05-09 — they were byte-identical to
the PROC- files now in this archive.

## Why kept (vs `git rm`)

These v1 entities are part of the project's history. Keeping them
under `archive/` (rather than deleting them) preserves the audit
trail for any agent or human looking back to understand why the
directories went away, and gives processkit a concrete reference
when authoring the v1→v2 Process migration that aibox was supposed
to emit (see `BACK-20260508_2241-QuietLark` C5 issue #4).
