---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0843-AmberThorn-aibox-should-emit-v1-v2-migration
  created: '2026-05-10T08:43:26+00:00'
  labels:
    version: v0.25.7-followup
    area: migration-emission
    github_issue: '72'
  updated: '2026-05-10T09:47:06+00:00'
spec:
  title: aibox should emit v1→v2 Migration entities on cutover releases (gh#72)
  state: done
  type: task
  priority: high
  description: |
    ## Source

    GitHub issue: https://github.com/projectious-work/aibox/issues/72

    ## Problem

    When upstream processkit supersedes a v1 entity type with a v2 primitive, the next aibox release should emit a Migration entity into derived projects so legacy v1 content gets explicitly addressed (transformed, archived, or accepted-as-historical) instead of silently lingering.

    Today only ONE v1→v2 content migration exists across all aibox releases (`MIG-20260503_164619-legacy-model-to-artifact-model-spec`). Other v2 boundaries shipped without any migration:

    - v1 `Actor` → v2 `TeamMember` (DEC-SpryTulip 2026-04-22). `context/actors/` orphaned in derived projects.
    - v1 `Process` → v2 `Scope+Gate+process-instance`. `context/processes/` orphaned.
    - v1 `StateMachine` → v2 lifecycle metadata. Empty `state-machines/` observed.

    Concrete impact: aibox @ v0.25.6 release agent re-planned twice because `route_task("release process")` returned `PROC-release` (v1) as authoritative instead of `NOTE-20260411` (the live process). An explicit migration would have moved that out months earlier.

    ## Proposed fix (per the issue)

    For each upstream processkit release that ships a v2 primitive replacing a v1 entity type, the aibox CLI release pipeline (likely `cli/src/migrate.rs` or where runtime-cleanup architecture emits Migration entities — see GentleFern's `cli/src/runtime_sync.rs::write_drift_migration_document` for the recent Variant 3 pattern) emits a Migration entity with:

    - `from_version` / `to_version` reflecting the upstream cutover release.
    - `summary` describing what v1 directory is superseded, by what v2 primitive, citing the upstream DEC.
    - `body` listing affected files, proposed transformation (migrate / archive / no-op), and instructions for `apply_migration`.

    On `aibox apply` against an old lock, the CLI surfaces the new pending migration in the apply preflight. `aibox apply` does NOT delete v1 entities silently — the Migration is the audit trail.

    ## Reference pattern

    `MIG-20260508-v0-25-6-lockfile-schema-bump` (commit e0ee7bc) is the textbook example. Apply the same pattern to every v1→v2 transition.

    ## Backfill scope (separate decision)

    The issue surfaces three known v1→v2 boundaries (Actor → TeamMember, Process → Scope+Gate, StateMachine → lifecycle) that shipped without migrations. Question: does this WorkItem also backfill those three Migration entities for the dogfood project, or only ship the emission **mechanism** for future cutovers?

    Recommendation: ship the mechanism in this WorkItem. Backfill the three known cutovers as a separate follow-up after the agent's mechanism lands and is reviewed.

    ## Files likely touched

    - `cli/src/migrate.rs` — the migration emitter for v1→v2 cutovers.
    - `cli/src/runtime_sync.rs` — coordination with existing Variant 3 emission pattern (refactor common helper if appropriate).
    - `cli/src/container.rs::cmd_apply` preflight — surfacing pending migrations to the user.
    - A v1→v2 catalog: hardcoded list of known cutover transitions per upstream release? Or a catalog file (e.g. `context/skills/processkit/_v1_v2_catalog.yaml`)?

    ## Acceptance

    - aibox CLI has a code path that emits a v1→v2 Migration on apply when the lock crosses a relevant upstream cutover.
    - Unit test exercises emission for at least one synthetic cutover.
    - `aibox apply` preflight lists pending Migrations including the new one.
    - Pattern matches the lockfile-schema-bump migration shape.

    ## PR commit message convention

    Reference `Closes #72` or `Fixes #72` in the commit body so GitHub auto-closes on merge.
  started_at: '2026-05-10T09:46:30+00:00'
  completed_at: '2026-05-10T09:47:06+00:00'
---

## Transition note (2026-05-10T09:47:06+00:00)

Implemented and merged in commit 7d70818 + merge de8f099 (closed gh#72). New cli/src/v1_v2_migration.rs ships CutoverDescriptor catalog + emit_v1_v2_migrations() (currently empty, mechanism only); wired into apply preflight in container.rs. 6 new tests. Backfill of 3 known historical cutovers tracked as BACK-EagerSea follow-up.
