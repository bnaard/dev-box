---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0946-EagerSea-v1-v2-migration-backfill-register-actor
  created: '2026-05-10T09:46:49+00:00'
  labels:
    version: v0.25.7-followup
    area: migration-emission
spec:
  title: 'v1→v2 Migration backfill: register Actor→TeamMember, Process→Scope+Gate,
    StateMachine→lifecycle in V1_TO_V2_CUTOVERS catalog'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Background

    AmberThorn (commit `7d70818`, merge `de8f099`, closes gh#72) shipped the v1→v2 Migration emission mechanism in `cli/src/v1_v2_migration.rs`. The compile-time `V1_TO_V2_CUTOVERS` catalog ships **empty** — the WorkItem was scoped to the mechanism only.

    This follow-up backfills the three known historical cutovers from upstream processkit so derived projects with orphaned v1 directories receive a pending Migration on next `aibox apply`.

    ## Cutovers to register

    Each entry is one `CutoverDescriptor` const in `V1_TO_V2_CUTOVERS`:

    1. **Actor → TeamMember**
       - upstream cutover release: ~processkit v0.18.x (verify with `git log` against `context/.processkit-provenance.toml` history or the SpryTulip DEC date)
       - v1 dir: `context/actors/`
       - v2 dir: `context/team-members/`
       - dec_ref: `DEC-20260422_0233-SpryTulip`
       - transformation_hint: `migrate` (each Actor maps to a TeamMember; if no clear mapping, archive)

    2. **Process → Scope+Gate**
       - upstream cutover release: ~processkit v0.12–0.15 (verify via processkit changelog / DECs)
       - v1 dir: `context/processes/`
       - v2: composition (`context/scopes/` + `context/gates/` + process-instance entities)
       - dec_ref: TBD (search for the v2 process composition decision; likely under context/decisions/ in processkit)
       - transformation_hint: `archive` (v1 Process docs are typically reference-only by the v2 cutover)

    3. **StateMachine → lifecycle metadata**
       - upstream cutover release: TBD
       - v1 dir: `context/state-machines/`
       - v2: lifecycle metadata embedded in entity schemas
       - dec_ref: TBD
       - transformation_hint: `archive` (state-machines were docs; the v2 lifecycle is enforced in code)

    ## Steps

    1. Verify each upstream release version (use `gh api` against the processkit repo or scan local processkit skill files for cutover decisions).
    2. Add three `CutoverDescriptor` const entries in `V1_TO_V2_CUTOVERS`.
    3. Add a unit test per entry exercising emission against a synthetic project with the v1 dir present.
    4. Smoke-test in dogfood: run `aibox apply` and confirm the apply preflight surfaces the new pending migrations (since the dogfood project shipped past those cutovers, depending on how it constructs its lock comparison, may or may not emit — check both fresh and old-lock paths).

    ## Acceptance

    - 3 cutovers in catalog with verified upstream release versions.
    - Unit tests pass.
    - Manual smoke-test: simulate a stale lock + populate `context/actors/` (or `context/processes/`) → preflight surfaces a pending Migration with the right shape.

    ## Refs

    - BACK-AmberThorn (predecessor — the mechanism)
    - gh#72 (closed)
    - DEC-20260422_0233-SpryTulip (Actor→TeamMember v2 cutover decision)
    - File: `cli/src/v1_v2_migration.rs` (catalog location)
---
