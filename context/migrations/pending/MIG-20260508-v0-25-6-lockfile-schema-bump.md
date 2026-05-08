---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260508-v0-25-6-lockfile-schema-bump
  created: 2026-05-08 00:00:00+00:00
  updated: '2026-05-08T00:00:00+00:00'
spec:
  source: aibox
  from_version: v0.25.5
  to_version: v0.25.6
  state: pending
  generated_by: manual
  generated_at: 2026-05-08 00:00:00+00:00
  summary: >-
    Extend aibox.lock with [addons.<name>.previous_selection] map and
    [harnesses].previous_selection set for cross-version managed-runtime
    cleanup (BR-CLEANUP-ARCH item 1, commit e0ee7bc).
---

# Migration MIG-20260508-v0-25-6-lockfile-schema-bump

From aibox `v0.25.5` to `v0.25.6` — lockfile schema bump.

## What changed

Commit `e0ee7bc` (`feat(cleanup-arch): lockfile schema bump + cross-version recognizer`)
extends `aibox.lock` with two new optional fields:

### `[addons].previous_selection`

`AddonsLockSection` gains a `previous_selection` field
(`BTreeMap<String, BTreeSet<String>>`). It records which tool names were
enabled under each addon family at the time of the previous `aibox apply`.
On the next apply, the host CLI diffs the current selection against this
map and can purge tool binaries that were baked into an earlier image layer
but are no longer enabled.

```toml
[addons]
resolved_at = "2026-05-08T12:00:00Z"

[addons.tools]
# resolved tool versions (existing field)
bat = "0.24.0"

[addons.previous_selection]
# NEW — populated automatically on first v0.25.6 apply
audio-voice = ["aider"]
git-ui      = ["lazygit"]
```

The field is serialized only when non-empty (`skip_serializing_if =
"BTreeMap::is_empty"`), so pre-existing lockfiles remain byte-identical
until the next apply runs.

### `[harnesses].previous_selection`

A new top-level `[harnesses]` section (`HarnessLockSection`) records the
set of AI harness names that were active at the time of the previous apply:

```toml
[harnesses]
recorded_at = "2026-05-08T12:00:00Z"
previous_selection = ["claude", "codex"]
```

This lets `aibox apply` compute a removal diff when a harness is
subsequently disabled — e.g., detecting stale `.mcp.json` entries or
harness-specific runtime files to purge. The field is gated on
`[apply].purge_disabled_harness_state` (default `false`); when `false`,
a pending migration entry is emitted instead of deleting state.

### Cross-version recognizer

`runtime_sync.rs` is extended to recognise the historical v0.25.5
managed-tmux file and the v0.25.3 `off_RIGHT` corruption signature.
Files matching these patterns are replaced automatically instead of
blocking apply.

## Backward compatibility

- Existing v0.25.5 lockfiles **parse cleanly** — both new sections are
  `#[serde(default, skip_serializing_if = "…")]`, so they are absent from
  old locks and the Deserializer silently fills in defaults.
- There are **no removals** from the lock schema; this is an additive bump.

## What derived projects need to do

1. **Run `aibox apply`** from a v0.25.6 host CLI. The CLI backfills
   `previous_selection` in both sections automatically on the first apply
   against an old lock — no manual editing is required.
2. **Verify** the bump by reading `aibox.lock` after apply:
   - `[aibox].cli_version` should be `"0.25.6"`.
   - `[addons].previous_selection` should be present and non-empty if any
     addon tools are enabled.
   - `[harnesses].previous_selection` should appear if any harnesses are
     active.
3. **No rebuild is needed** solely for the lockfile change — the schema
   lives entirely in host-side Rust, not in the container image.

See `cli/src/lock.rs` for the canonical field definitions
(`AddonsLockSection::previous_selection`, `HarnessLockSection`).
