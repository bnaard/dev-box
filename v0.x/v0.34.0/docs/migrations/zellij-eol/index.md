# 

# Zellij end-of-life migration (v0.25.5 → v0.25.6)

## TL;DR

aibox v0.25.6 removes Zellij entirely. tmux is now the only supported
terminal multiplexer. Any `[customization.zellij_status]` section in your
`aibox.toml` causes schema validation to hard-reject `aibox apply` — you
must remove it before upgrading. Stale Zellij directories and binaries are
purged automatically on the first `aibox apply` that runs against a
v0.25.6+ host CLI. No data from active work sessions is touched; only
Zellij runtime artifacts are removed.

## What changed

The following items are removed or rejected in v0.25.6 (commit `faa9a88`,
decision `DEC-20260508_1515-SilentAsh`):

- **`[customization.zellij_status]` config key** — the field is removed
  from the `Customization` struct in `cli/src/config.rs`. The TOML
  deserializer now hard-rejects any `aibox.toml` that still contains this
  section, with a descriptive error pointing to this document.
- **`--forget-zellij-state` CLI flag** — removed from the argument parser
  in `cli/src/cli.rs`. Scripts or aliases that reference this flag will
  fail to parse.
- **Unconditional purge on `aibox apply`** — the following paths under
  `.aibox-home/` are deleted on every apply regardless of config:
  - `.config/zellij/`
  - `.cache/zellij/`
  - `.local/share/zellij/`
  - `.local/bin/aibox-status` (the shell-backed Zellij helper; superseded
    by the tmux PowerKit plugin set)

  The purge is performed by `cleanup_legacy_zellij_files()` in
  `cli/src/seed.rs`, which calls unconditionally via `LEGACY_MUX_RELPATHS`.
- **`aibox doctor` errors** — any surviving artifact from the list above
  triggers an `ERROR` diagnostic (`check_legacy_zellij_artifacts` in
  `cli/src/doctor.rs`). The error is not advisory; it blocks a clean
  doctor run.

## What you need to do

Complete these steps **on the host** before or immediately after upgrading
to v0.25.6:

a. **Remove `[customization.zellij_status]`** from your `aibox.toml`.
   Open the file and delete the entire section (header and all keys beneath
   it). If you have no such section, skip this step.

b. **Migrate any custom status configuration to tmux.** If you previously
   used Zellij status customizations, set the tmux equivalent in `aibox.toml`:

   ```toml
   [customization.tmux.status]
   mode = "extended"   # or "minimal" for a compact single-line bar
   ```

   The `extended` mode renders a two-line powerline bar with aibox metrics
   (log/OOM/proc/AI/MCP/migration counters). The `minimal` mode renders a
   single line. See `docs-site/content/docs/customization/layouts.md` for full
   reference.

c. **Run `aibox apply`** from a v0.25.6+ host CLI. This purges the stale
   Zellij artifacts listed above and records the new lockfile schema fields.

d. **Verify with `aibox doctor`.** After apply, run:

   ```
   aibox doctor
   ```

   A passing run reports no `check_legacy_zellij_artifacts` errors. If
   artifacts survive (e.g., because a volume mount shadowed the purge),
   the error output lists the exact paths to remove manually.

## Why

Zellij was introduced as an aibox sidecar multiplexer, but the WASM plugin
runtime, session-state model, and config schema diverged frequently from
aibox's tmux-native layout engine. The persistent vim-pane handoff through
Zellij regressed every three to five releases, and the native Zellij status
plugin required a WASM build step that added both CI complexity and binary
supply-chain surface. tmux has been the canonical aibox multiplexer since
v0.25.0; keeping a Zellij compatibility layer alongside it caused drift in
every layout-generation codepath.

Decision `DEC-20260508_1515-SilentAsh` records the full rationale and the
choice of scorched-earth excision (Variant 1 hard-purge) over a softer
deprecation path.

## Need help

Open an issue at <https://github.com/projectious-work/aibox/issues> and
tag it `zellij-migration`. Include the output of `aibox doctor` and the
relevant section of your `aibox.toml`.


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.0/docs/migrations/zellij-eol/index.md
