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
  updated: '2026-08-21T19:58:56+00:00'
spec:
  title: 'v0.25.6: Stale-process / v1-legacy cleanup — processes/ dups, deprecation
    banners, processes+actors archival, upstream processkit issues'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration
    full text.
  started_at: '2026-05-08T22:47:19+00:00'
  completed_at: '2026-08-21T19:58:56+00:00'
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


## Transition note (2026-08-21T19:58:56+00:00)

Closed from the stale review queue by owner direction on 2026-08-21; subsequent v0.x releases supersede this historical review item.
