---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0951-TallStream-processkit-v0-25-8-v0-26
  created: '2026-05-10T09:51:55+00:00'
  labels:
    version: v0.25.7-integration
    area: processkit-upgrade
  updated: '2026-05-10T11:24:00+00:00'
spec:
  title: 'processkit v0.25.8 → v0.26.0 integration: sync + source-upgrade migration
    + new MCP vocabulary'
  state: done
  type: task
  priority: high
  description: |
    ## Background

    Upstream processkit shipped v0.26.0 today (2026-05-10). This is a minor release with substantial additions: RoleSlot primitive (5 new MCP tools), catalog-driven `pk-team-create`, consultant TeamMember type, budget drift detection, route_task response field extensions, compliance contract rewrite, slim per-turn hook, and lazy-import for aggregate-mcp (resolves processkit#31).

    The aibox v0.25.7 release should integrate v0.26.0 before shipping.

    ## Goal

    Run the full processkit upgrade end-to-end and add aibox-side awareness for the new vocabulary.

    ## Scope

    ### A. Sync + version bump
    1. Run `./scripts/maintain.sh sync-processkit` to bump `PROCESSKIT_DEFAULT_VERSION` in `cli/src/processkit_vocab.rs` from `v0.25.8` to `v0.26.0`.
    2. Review the FORMAT.md diff surfaced by sync-processkit (handle any vocabulary changes — new categories, renamed dirs, new filename constants).
    3. Run `cargo test processkit_vocab` to confirm no obvious drift.

    ### B. Source-upgrade Migration
    4. Run `aibox apply` (or whatever invokes the processkit-content sync). This should generate a new Migration entity in `context/migrations/pending/` like `MIG-…-v0.25.8-to-v0.26.0.md` describing the upstream diff (changed/conflicts/new/removed).
    5. Review the generated Migration body. Resolve any conflicts in customized aibox skills.
    6. Apply the Migration via `mcp__processkit-gateway__apply_migration`.

    ### C. New v0.26.0 MCP vocabulary additions in processkit_vocab.rs

    Add these to the appropriate constants/lists in `cli/src/processkit_vocab.rs` (study the file structure first; vocabulary is grouped by domain — workitem, decision, etc.):

    **5 RoleSlot tools** (new domain group `role_slot` likely):
    - `create_role_slot`
    - `get_role_slot`
    - `list_role_slots`
    - `fill_role_slot`
    - `close_role_slot`

    **Budget tool** (likely under decision-record or scope-management domain):
    - `query_budget_drift`

    **route_task response fields** (extends the route_task return shape):
    - `recommended_team_member_slug`
    - `recommended_model_class`

    If these vocab additions exist on the upstream side already (delivered via the source-upgrade migration content), aibox just needs to add them to its `processkit_vocab.rs` constants so router/test code knows they exist. Verify whether they're route_task DOMAIN_GROUPS, MCP TOOL_CATALOG entries, or both.

    ### D. Compliance contract update
    7. v0.26.0 ships a rewritten `compliance-contract.md` (50 lines, 6 sections, BEGIN/END HOOK markers, sub-agent-dispatch clause). Review what the source-upgrade migration brought in. If aibox customized the prior contract, resolve.

    ### E. Hook sync
    8. v0.26.0 ships slim per-turn UserPromptSubmit hook + full SessionStart hook. aibox's hook configuration in `.claude/settings.json` (and similar for other harnesses) may need to reflect the new injection format. Inspect what the processkit content sync brought in; coordinate with `cli/src/seed.rs` if seed-time config changed.

    ### F. release-audit exemption update
    9. v0.26.0 release-audit exempts `migrations/applied/` and `migrations/rejected/` from v1 apiVersion errors. Our local `scripts/release-audit-stale-tests.py` (TallBear) already has exemption logic — verify it covers these two dirs; update if not.

    ### G. Migration schema relaxation
    10. v0.26.0 relaxed Migration `spec.required` to `[source, state]` only. Aibox's local Migration generation (e.g. `cli/src/runtime_sync.rs::write_drift_migration_document`, `cli/src/v1_v2_migration.rs::emit_v1_v2_migrations`) should still validate. Run any existing migration-emission tests to confirm.

    ## Acceptance

    - `processkit_vocab.rs` PROCESSKIT_DEFAULT_VERSION == v0.26.0.
    - New vocabulary entries present and tested.
    - The 0.25.8→0.26.0 source-upgrade Migration applied (moved to `context/migrations/applied/`).
    - `cargo check` clean; `cargo test --bin aibox` ≥ baseline.
    - `pk-doctor` (post-upgrade processkit v0.26.0 version) runs clean — ERRORs would block release.
    - `aibox doctor` runs clean.
    - Compliance hook configuration reflects v0.26.0 contract.

    ## Refs

    - Upstream release: https://github.com/projectious-work/processkit/releases/tag/v0.26.0
    - Sibling WorkItem: BACK-20260510_0325-DaringAsh (lazy-import wiring; will run in parallel)
    - File: `cli/src/processkit_vocab.rs`, `scripts/maintain.sh::cmd_sync_processkit`
  started_at: '2026-05-10T11:23:48+00:00'
  completed_at: '2026-05-10T11:24:00+00:00'
---

## Transition note (2026-05-10T11:24:00+00:00)

Implemented and merged in commits 4ee2d99 + 92f5d34 + merge 30d9eaf. Source-upgrade Migration MIG-20260510T100327 applied (28 changed, 0 conflicts, 48 new, 0 removed). 8 new vocabulary constants in cli/src/processkit_vocab.rs (5 RoleSlot tools, query_budget_drift, 2 route_task fields). Compliance contract v2 rewrite. 905 tests pass.
