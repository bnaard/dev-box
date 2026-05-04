---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260504_1242-CuriousPeak-upgrade-aibox-toml-through-safe-config
  created: '2026-05-04T12:42:33+00:00'
spec:
  title: Upgrade aibox.toml Through Safe Config Migrations
  state: accepted
  decision: When a newer aibox CLI sees an older aibox.toml, it should run an ordered,
    idempotent config-migration layer using a structure-preserving TOML editor. The
    migration layer may auto-apply clean changes such as missing sections, missing
    default keys, and unambiguous renames. Ambiguous or conflict-prone changes must
    not overwrite the file; instead aibox should generate reviewable migration guidance
    and exact TOML snippets or patch artifacts while continuing apply from an in-memory
    normalized config where safe.
  context: Derived projects can run a newer aibox CLI against an older aibox.toml
    shape. The CLI currently tolerates missing fields through defaults and some one-shot
    migrations, but this does not consistently update the committed configuration
    to the newer schema. Whole-file regeneration is unsafe because aibox.toml is user-owned,
    comment-heavy, and often locally customized.
  rationale: This keeps CLI upgrades from leaving projects on stale config schemas,
    while respecting aibox.toml as a declarative project-owned file. Structure-preserving
    edits reduce merge conflict risk, and explicit migration artifacts handle cases
    that require owner review.
  alternatives:
  - option: Regenerate aibox.toml wholesale during apply
    status: rejected
    reason: Would discard comments, ordering, and user edits, and would create large
      merge conflicts.
  - option: Only warn and never edit aibox.toml
    status: rejected
    reason: Leaves derived projects permanently stale unless every owner manually
      tracks schema changes.
  - option: Switch aibox.toml to another format as part of migration
    status: deferred
    reason: The current decision concerns safe schema evolution; format choice remains
      under review.
  consequences: aibox.lock should track applied config migrations so reruns are quiet.
    New aibox.toml schema changes need a migration entry and tests for both clean
    auto-apply and conflict/advisory paths. Full aibox.toml regeneration remains an
    init-only behavior, not an apply-time behavior.
  decided_at: '2026-05-04T12:42:33+00:00'
---
