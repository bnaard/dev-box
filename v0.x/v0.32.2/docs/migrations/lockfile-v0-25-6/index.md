# 

LLMS index: [llms.txt](/aibox/v0.x/v0.32.2/llms.txt)

---

# Lockfile schema bump v0.25.6 — what's automatic, what to verify

aibox v0.25.6 extends `aibox.lock` with two new optional sections. The
bump is fully automatic — `aibox apply` backfills the new fields on first
run. No manual editing is required, and existing lockfiles remain valid
(the new fields use `#[serde(default)]`, so they are absent from old locks
without causing parse errors).

## What changed

`cli/src/lock.rs` adds:

- **`AddonsLockSection::previous_selection`**
  (`BTreeMap<String, BTreeSet<String>>`) — records which tool names were
  enabled under each addon family at the time of the last apply. Written
  under `[addons.previous_selection]` in `aibox.lock`. Used on the next
  apply to compute a removal diff when a tool is disabled, so stale addon
  binaries baked into an earlier image layer can be purged cleanly.

- **`[harnesses]` section** (`HarnessLockSection`) — records the set of
  AI harness names that were active (`previous_selection:
  BTreeSet<String>`) and the timestamp when the record was taken
  (`recorded_at`). Used by `aibox apply` to detect harnesses that were
  active last time but are no longer configured, enabling targeted
  cleanup of harness-specific state files (gated on
  `[apply].purge_disabled_harness_state`, default `false`).

Both fields are populated automatically on the first `aibox apply` that
runs against an old v0.25.5 lockfile.

## Troubleshooting

**`[addons.previous_selection]` is absent after apply.**

This is expected if no addon tools are enabled — the field serializes
only when non-empty (`skip_serializing_if = "BTreeMap::is_empty"`). Enable
at least one tool under `[addons]` in `aibox.toml` and re-run
`aibox apply` to see it populated.

**`aibox apply` reports a lockfile parse error after upgrading from
v0.25.5.**

A truncated or hand-edited `aibox.lock` may have a malformed `[addons]`
section. Check that `[addons].resolved_at` is present and is a valid
ISO 8601 timestamp. If the file is corrupt, delete `aibox.lock` and run
`aibox apply` — the CLI regenerates it from scratch.
