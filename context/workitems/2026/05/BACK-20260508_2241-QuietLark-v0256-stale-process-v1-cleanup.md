---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2241-QuietLark-v0256-stale-process-v1-cleanup
  created: '2026-05-08T22:41:20+00:00'
  labels:
    track: context-hygiene
    release: v0.25.6
    deferred_via: DEC-20260508_2235-CuriousBadger
  updated: '2026-05-08T22:50:45+00:00'
spec:
  title: 'v0.25.6: Stale-process / v1-legacy cleanup — processes/ dups, deprecation
    banners, processes+actors archival, upstream processkit issues'
  state: review
  type: task
  priority: medium
  description: |
    ## Goal

    Eliminate the stale process descriptions and v1-legacy directories that misled the agent twice this session.

    ## Items

    ### C1 — Delete the 4 slug-named duplicate files in context/processes/
    - `context/processes/release.md` (identical to `PROC-release.md` per `diff -q`)
    - `context/processes/bug-fix.md` (vs `PROC-bug-fix.md`)
    - `context/processes/code-review.md` (vs `PROC-code-review.md`)
    - `context/processes/feature-development.md` (vs `PROC-feature-development.md`)
    - INDEX.md only references the PROC- prefixed files; the slug-named copies are silent stale aliases.

    ### C2 — Add a deprecation banner to PROC-release.md
    - Header at the top: "**DEPRECATED — see `context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` and `AGENTS.md:139` for the live aibox release process.** This v1 Process entity is retained for historical context only."
    - Same for the other three PROC- files if their content is also superseded — verify each before adding.

    ### C3 — Archive context/processes/ as a whole (if Sage agrees after audit)
    - The skill catalog marks `process-management` as "Legacy/migration guidance for v1 Process entities". v2 replaces v1 Process with Scope+Gate+process-instance composition.
    - Decide: move `context/processes/` to `context/archive/processes/` (or equivalent), keep INDEX.md visible at top of archive, and remove the `processes/` directory from the live tree.
    - Needs a fresh DEC if the answer is "archive whole directory".

    ### C4 — Audit context/actors/ (9 v1 entities)
    - Whole directory superseded by `team-members/` per DEC-20260422_0233-SpryTulip.
    - Decide: archive parallel to C3, or migrate the 9 actor entities to TeamMembers if any have signal worth preserving.

    ### C5 — File 5 upstream issues (3 processkit, 2 aibox) — expanded after root-cause analysis 2026-05-09

    **processkit (`projectious-work/processkit`):**
    1. `find_skill` / `task-router` should weight v1 entities downward when a v2 successor exists, so legacy entries don't show up as authoritative in routing results. (Original issue.)
    2. `pk-doctor` should grow a `v1_entity_drift` check that flags the count of v1 entities per directory and proposes migration. Same shape as the workitem-state-drift candidate from the prior session's behavioural retrospective. (Original issue.)
    3. `pk-doctor` SKILL.md should expose its full check inventory — `v2_contracts.py`, `context_hygiene.py`, `schema_vocabulary.py`, `migration_integrity.py`, `mcp_gateway.py`, `skill_dag.py` exist on disk but are NOT documented in the contract. Agents won't lean on undocumented checks. (NEW — found 2026-05-09 root-cause analysis.)

    **aibox (`projectious-work/aibox`):**
    4. When upstream processkit supersedes a v1 entity type with a v2 primitive (`actor-profile → team-manager`, `Process → Scope+Gate+instance`, etc.), the next aibox release MUST emit a Migration entity in derived projects so legacy v1 content gets explicitly addressed instead of silently lingering. Today, only `MIG-20260503_164619-legacy-model-to-artifact-model-spec` exists — the actor and process v2 boundaries shipped without migrations. (NEW.)
    5. Phase 0 of `NOTE-20260411-aibox-release-process` should run `pk-doctor` AND `aibox doctor` and surface findings BEFORE `bump-version`. Both doctors exist; neither is currently in the release ritual. (NEW.)

    Use `gh issue create --repo <repo>` for each.

    ## Dispatch hint
    - C1, C2: Robin (junior eng / mechanical) — file deletions + banner adds.
    - C3, C4: Sage (CTO) for the architectural call, then Cora records the DECs, then Robin executes the move.
    - C5: Jordan (technical writer) for the issue body, Cora files via gh.

    ## Why C5 expanded — root-cause analysis 2026-05-09

    The agent re-planned the v0.25.6 release twice this session because PROC-release misled it. Investigation showed three independent safety-net failures:
    1. **aibox** did not emit migrations for the v1→v2 entity-type transitions (actor, process, state-machine). Only one v1→v2 content migration was ever recorded.
    2. **aibox doctor** explicitly delegates `context/` checks to processkit (`cli/src/doctor.rs:38-45`).
    3. **pk-doctor** has no documented check for v1-entity drift. The undocumented `v2_contracts.py` / `context_hygiene.py` modules exist but agents won't invoke checks they can't see in the SKILL.md contract.

    Plus a process miss: neither doctor is gated to the canonical release ritual, so even working safety nets wouldn't fire before tag.

    ## Acceptance
    - `context/processes/*.md` slug-named duplicates gone.
    - Live PROC- files carry deprecation banners.
    - C3/C4 either executed or formally re-deferred via fresh DEC.
    - Both upstream issues filed with URLs recorded in v0.25.6 release notes.
  started_at: '2026-05-08T22:47:19+00:00'
---

## Transition note (2026-05-08T22:47:19+00:00)

Robin starting the cleanup. C5 expanded to 5 issues per 2026-05-09 root-cause analysis (logged separately). Archival DEC for processes/ + actors/ recorded in this batch.


## Transition note (2026-05-08T22:50:45+00:00)

Cleanup complete. Summary:

C1 — DONE. 4 slug-named duplicates `git rm`'d (release.md, bug-fix.md, code-review.md, feature-development.md).

C2 — DROPPED (no longer applicable). Banners-on-PROC-files made moot by C3 archival.

C3 — DONE per DEC-20260508_2247-FierceQuail. `context/processes/` (7 files including INDEX, 4 PROC-, team-task-distribution) moved to `context/archive/processes-v1/` with a redirect README.

C4 — DONE per DEC-20260508_2247-FierceQuail. `context/actors/` (9 v1 entities — Bernhard duplicate + 8 role templates) moved to `context/archive/actors-v1/` with a redirect README.

C5 — DONE. 5 GitHub issues filed (3 processkit, 2 aibox) — expanded from 2 after the 2026-05-09 root-cause analysis:
- processkit#21 (find_skill/task-router v1 down-weight): https://github.com/projectious-work/processkit/issues/21
- processkit#22 (pk-doctor v1_entity_drift check): https://github.com/projectious-work/processkit/issues/22
- processkit#23 (pk-doctor SKILL.md check inventory doc-gap): https://github.com/projectious-work/processkit/issues/23
- aibox#72 (v1→v2 Migration emission): https://github.com/projectious-work/aibox/issues/72
- aibox#73 (Phase 0 doctor invocation): https://github.com/projectious-work/aibox/issues/73

Acceptance:
- ✓ slug-named duplicates gone
- ✓ live PROC- files archived (banners moot)
- ✓ C3/C4 executed (DEC-20260508_2247)
- ✓ all 5 issues filed; URLs above

Ready for owner review.
