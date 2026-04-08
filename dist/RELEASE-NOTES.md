# aibox v0.16.5 — MCP server registration + render-mirror diff + skills filter + python/uv in base

Bundle release with four interlocking changes that all touch the
install/sync/diff machinery, plus a small zellij layout fix.

## 1. MCP server registration (DEC-033 / BACK-119)

aibox now writes per-harness MCP server registration files at the
project root, walking the templates mirror at
`context/templates/processkit/<version>/skills/*/mcp/mcp-config.json`
and dispatching per `[ai].providers`:

| Provider | File written | Translator |
|---|---|---|
| `Claude` OR `Mistral` | `.mcp.json` | none (Claude shape) |
| `Cursor` | `.cursor/mcp.json` | none (same shape) |
| `Gemini` | `.gemini/settings.json` | none (same shape, preserves other top-level keys) |
| `Codex` | `.codex/config.toml` | JSON → TOML |
| `Continue` | `.continue/mcpServers/processkit-<name>.json` (per file) | per-file flat shape |
| `Aider` | (no file written; sync warns Aider has no built-in MCP client) | n/a |

Three writers cover the five active providers because Claude Code,
Cursor, and Gemini CLI all use the **identical**
`{"mcpServers": {"<name>": {"command", "args", "env"}}}` JSON shape.
Two real translators: `write_codex_config_toml` (JSON spec →
`[mcp_servers.<name>]` TOML sections via `toml_edit`),
`write_continue_mcp_dir` (per-server file writer).

### Mistral routing

Mistral has MCP client capability via Python SDK and Le Chat custom
connectors but no local file-based project config. When `Mistral` is
in `[ai].providers`, aibox writes `.mcp.json` (the standard Claude
shape) so a custom Mistral SDK-based CLI tool the user might build
can read MCP server registrations from the conventional location.
If both Claude and Mistral are listed, `.mcp.json` is written once.

### Non-destructive merge

Each writer reads the existing harness file (if present), removes
entries whose key is in the **managed set**, and adds the current
entries. The managed set members are the **JSON keys** from each
per-skill `mcp-config.json`'s `mcpServers` object — i.e., the
prefixed names processkit ships
(`processkit-workitem-management`, `processkit-decision-record`,
…). User-added entries with names outside the managed set survive
every sync untouched.

A regression test
(`collect_keys_managed_set_on_mcpservers_json_key_not_directory_name`)
locks in the JSON-key-vs-directory-name distinction so no future
refactor can swap them — paired with
[`projectious-work/processkit#2`](https://github.com/projectious-work/processkit/issues/2)
where processkit confirmed the prefix has shipped since v0.5.0.

### New `[ai].providers` values

`Cursor`, `Codex`, and `Continue` are added to the `AiProvider` enum.
They are pure file-write triggers — no in-container persistence,
no thin-pointer markdown, no addon. They influence MCP registration
file writes and the sync perimeter only.

### Effective skill set integration

`regenerate_mcp_configs` walks the same effective skill set as the
install path (DEC-035 below). A skill that's filtered out by
`[skills].include`/`exclude` doesn't get its MCP server registered.

## 2. Rendered templates mirror (DEC-034)

v0.16.4 shipped templated installs (`scaffolding/AGENTS.md` rendered
through the Class A vocabulary) but the templates mirror held the
unrendered cache content, causing SHA-based diff comparison to
always false-positive as "ChangedLocally". v0.16.4's workaround was
to skip templated files from the diff entirely (BACK-119 known
limitation).

v0.16.5 fixes it: `copy_templates_from_cache_with_vars` walks each
file through `install_action_for`, and `InstallTemplated` files are
read → rendered with the same Class A vocabulary the live install
uses → written into the mirror. The mirror's SHA now matches the
live SHA on a fresh install. `content_diff` drops its
`InstallTemplated → Skip` arms; templated files are now classified
correctly. User edits to AGENTS.md are surfaced as
"ChangedLocallyOnly" instead of being silently ignored.

**Limitation NOT fixed in v0.16.5:** templated files keep
`write_if_missing` semantics in the install path. Cross-version
upgrades still don't auto-propagate template improvements to
already-existing AGENTS.md. The full pre-install three-way merge
fix is queued as **BACK-120** for a future release.

## 3. `[skills].include` / `[skills].exclude` activation (DEC-035 / BACK-118)

The `[skills]` section parsed but was no-op through v0.16.4. v0.16.5
makes it functional:

1. **Effective skill set computation**: walk the templates mirror
   at `context/templates/processkit/<version>/packages/` for each
   selected `[context].packages`, recursively expand `extends:`
   (cycle protection via visited-set), take the union of every
   package's `spec.includes.skills` list, then add `[skills].include`
   and remove `[skills].exclude`.
2. **Install path filters**: skill files under `skills/<name>/`
   where `<name>` is not in the effective set are skipped (counted
   as `files_skipped`). The `_lib/` shared MCP server lib is never
   filtered.
3. **MCP registration filters by the same set** so a skill that's
   not installed also doesn't get its MCP server registered.
4. **`aibox doctor` validates** `[skills]` overrides — warns if any
   name in `include`/`exclude` is not a known processkit skill (typo
   detection).

**First-install special case**: the effective set requires the
templates mirror, which doesn't exist on the FIRST install. For the
first install the filter is `None` and every skill is installed;
filtering takes effect from the SECOND sync onward.

7 new content_init unit tests covering single package, extends
chain, diamond inheritance, user include, user exclude, version
unset, and unknown package error.

## 4. python3 + uv in base-debian image (DEC-036)

`python3` (Debian Trixie default — 3.13.x) and `uv` (latest from
`ghcr.io/astral-sh/uv:latest` via `COPY --from=`) are now baked into
the base-debian image unconditionally. processkit's MCP servers are
PEP 723 scripts that run via `uv run script.py` — they need both
present in the container regardless of whether the user enabled the
python addon. v0.16.4 gated them on the addon and bit any user who
skipped it ("MCP servers fail to launch" with no obvious fix).

The python addon (`addons/languages/python.yaml`) still ships but
its purpose shifts from "Python at all" to "additional Python
tooling beyond the base minimum" (poetry, pdm, alternative Python
versions). Description updated.

Size impact: ~75-100 MB (~10% growth on a ~800 MB base). First sync
after upgrade pulls the new base image; subsequent syncs unchanged.

`context/work-instructions/RELEASE-PROCESS.md` Phase 0 dependency
table extended with `python3` and `uv`.

## 5. Sync perimeter additions

Five new entries in `SYNC_PERIMETER`, gated dynamically per
`[ai].providers` at write time:

- `.mcp.json` (Claude Code, Mistral SDK consumers)
- `.cursor/` (Cursor)
- `.gemini/` (Gemini CLI)
- `.codex/` (Codex CLI)
- `.continue/` (Continue)

The static perimeter lists them all because it's a contract about
which paths sync MAY touch, not which it WILL touch on this run.
Tests:
- `mcp_registration_files_are_in_perimeter` covering all five paths
- `provider_mcp_directories_are_in_perimeter_v0_16_5` flipping the
  v0.16.4 "all provider dirs are out" assertion for the new four
- `claude_internal_files_are_out_of_perimeter` clarifying that
  `.claude/` itself stays out (Claude Code's MCP file lives at
  the project root, not under `.claude/`)
- `all_known_sync_write_targets_are_in_perimeter` extended with the
  five new write targets

## 6. Zellij ai-layout 50/50 split (slipped in)

`generate_ai_layout` in `cli/src/seed.rs` now uses a 50/50 horizontal
split on screen 1 (yazi files / AI agent pane) instead of the
previous 53/47. Tiny user-requested fix bundled with v0.16.5.

## Quality gates

- **495/495 tests pass** (was 467 in v0.16.4) — 28 new tests:
  - 18 in `mcp_registration` (parser, managed-set computation,
    each writer's merge logic, JSON-key-vs-directory-name regression)
  - 7 in `content_init` for `build_effective_skill_set` (single
    package, extends chain, diamond, user include, user exclude,
    unset version, unknown package error)
  - 2 in `sync_perimeter` for the new MCP perimeter entries
  - 1 in `seed` updated for the 50/50 split
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo audit` clean

## Migration impact

**Mostly backwards compatible.** No config schema changes, no CLI
breaking changes. Existing v0.16.4 projects pick up the changes on
next `aibox sync` after upgrading the binary AND pulling the new
base image (Phase 2 of the release):

- **MCP files appear** at the next sync, depending on
  `[ai].providers`. User-added entries in any pre-existing
  `.mcp.json` are preserved (non-destructive merge by JSON key).
- **`[skills]` filtering** takes effect (was no-op before). Today
  no projects use these fields, so no observable change.
- **Templated files (AGENTS.md)** become diffable. User edits to
  AGENTS.md were silently ignored by v0.16.4's diff; v0.16.5
  surfaces them as `ChangedLocallyOnly`. No file content changes.
- **Base image grows ~10%** due to python3 + uv. First sync after
  Phase 2 triggers a fresh image pull; subsequent syncs cached.

## What's next (v0.16.6+)

- **BACK-120**: pre-install three-way diff to fix the
  edit-clobbering footgun on processkit version upgrades.
- **BACK-121**: track Aider native MCP client support upstream.

## Linked decisions

- **DEC-036** — python3 + uv unconditionally in base-debian image
- **DEC-035** — `[skills].include` / `[skills].exclude` activation
- **DEC-034** — Render templated files into the templates mirror
- **DEC-033** — Per-harness MCP server registration
- **DEC-032** — Class A placeholder vocabulary for templated installs
- **DEC-031** — xdg-open shim in base image
- **DEC-030** — three-tier privacy model
- **DEC-029** — list_versions GitHub fallback + sync perimeter catch-up
- **DEC-028** — sync auto-installs processkit, init picks the version
- **DEC-027** — aibox v0.16.0: rip the bundled process layer
- **DEC-026** — Cache-tracked processkit reference
- **DEC-025** — Generic content-source release-asset fetcher
