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
  title: "v1\u2192v2 Migration backfill: register Actor\u2192TeamMember, Process\u2192Scope+Gate, StateMachine\u2192lifecycle in V1_TO_V2_CUTOVERS catalog"
  state: backlog
  type: task
  priority: medium
  description: "## Background\n\nAmberThorn (commit `7d70818`, merge `de8f099`, closes gh#72) shipped the v1\u2192v2 Migration emission mechanism in `cli/src/v1_v2_migration.rs`. The compile-time `V1_TO_V2_CUTOVERS` catalog ships **empty** \u2014 the WorkItem was scoped to the mechanism only.\n\nThis follow-up backfills the three known historical cutovers from upstream processkit so derived projects with orphaned v1 directories receive a pending Migration on next `aibox apply`.\n\n## Cutovers to register\n\nEach entry is one `CutoverDescriptor` const in `V1_TO_V2_CUTOVERS`:\n\n1. **Actor \u2192 TeamMember**\n   - upstream cutover release: ~processkit v0.18.x (verify with `git log` against `context/.processkit-provenance.toml` history or the SpryTulip DEC date)\n   - v1 dir: `context/actors/`\n   - v2 dir: `context/team-members/`\n   - dec_ref: `DEC-20260422_0233-SpryTulip`\n   - transformation_hint: `migrate` (each Actor maps to a TeamMember; if no clear mapping, archive)\n\n2. **Process\
    \ \u2192 Scope+Gate**\n   - upstream cutover release: ~processkit v0.12\u20130.15 (verify via processkit changelog / DECs)\n   - v1 dir: `context/processes/`\n   - v2: composition (`context/scopes/` + `context/gates/` + process-instance entities)\n   - dec_ref: TBD (search for the v2 process composition decision; likely under context/decisions/ in processkit)\n   - transformation_hint: `archive` (v1 Process docs are typically reference-only by the v2 cutover)\n\n3. **StateMachine \u2192 lifecycle metadata**\n   - upstream cutover release: TBD\n   - v1 dir: `context/state-machines/`\n   - v2: lifecycle metadata embedded in entity schemas\n   - dec_ref: TBD\n   - transformation_hint: `archive` (state-machines were docs; the v2 lifecycle is enforced in code)\n\n## Steps\n\n1. Verify each upstream release version (use `gh api` against the processkit repo or scan local processkit skill files for cutover decisions).\n2. Add three `CutoverDescriptor` const entries in `V1_TO_V2_CUTOVERS`.\n\
    3. Add a unit test per entry exercising emission against a synthetic project with the v1 dir present.\n4. Smoke-test in dogfood: run `aibox apply` and confirm the apply preflight surfaces the new pending migrations (since the dogfood project shipped past those cutovers, depending on how it constructs its lock comparison, may or may not emit \u2014 check both fresh and old-lock paths).\n\n## Acceptance\n\n- 3 cutovers in catalog with verified upstream release versions.\n- Unit tests pass.\n- Manual smoke-test: simulate a stale lock + populate `context/actors/` (or `context/processes/`) \u2192 preflight surfaces a pending Migration with the right shape.\n\n## Refs\n\n- BACK-AmberThorn (predecessor \u2014 the mechanism)\n- gh#72 (closed)\n- DEC-20260422_0233-SpryTulip (Actor\u2192TeamMember v2 cutover decision)\n- File: `cli/src/v1_v2_migration.rs` (catalog location)"
---
