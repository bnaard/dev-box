//! aibox ↔ processkit compatibility table.
//!
//! Each entry maps an exact aibox CLI version to the processkit version it
//! was released with and tested against. This is the MINIMUM compatible
//! processkit version for that aibox release.
//!
//! When a project's `[processkit].version` in `aibox.toml` is older than
//! the minimum for the running aibox binary, `aibox apply` emits a warning.
//!
//! Update this table with every aibox release that changes processkit
//! compatibility. Keep entries in ascending version order.

/// One entry in the compatibility table.
pub struct CompatEntry {
    /// The exact aibox release version.
    pub aibox_version: &'static str,
    /// The processkit version this aibox was released with (minimum compatible).
    pub processkit_version: &'static str,
    /// Brief note on what changed in processkit at this boundary.
    pub note: &'static str,
}

/// Compatibility table: aibox version → minimum processkit version.
///
/// If your aibox version is not listed, use the entry for the closest
/// older listed version.
pub static COMPAT_TABLE: &[CompatEntry] = &[
    CompatEntry {
        aibox_version: "0.16.0",
        processkit_version: "v0.4.0",
        note: "initial processkit integration",
    },
    CompatEntry {
        aibox_version: "0.16.1",
        processkit_version: "v0.4.0",
        note: "sync auto-install added",
    },
    CompatEntry {
        aibox_version: "0.17.0",
        processkit_version: "v0.5.0",
        note: "aibox.lock sectioned format (DEC-037)",
    },
    CompatEntry {
        aibox_version: "0.17.2",
        processkit_version: "v0.6.0",
        note: "core skill enforcement, processkit v0.6.0 compat",
    },
    CompatEntry {
        aibox_version: "0.17.3",
        processkit_version: "v0.6.0",
        note: "Claude Code slash-command adapters (aibox#37)",
    },
    CompatEntry {
        aibox_version: "0.17.4",
        processkit_version: "v0.6.0",
        note: "content migration documents (pending/in-progress/applied)",
    },
    CompatEntry {
        aibox_version: "0.17.5",
        processkit_version: "v0.8.0",
        note: "processkit v0.8.0 GrandLily src/ restructure",
    },
    CompatEntry {
        aibox_version: "0.17.6",
        processkit_version: "v0.8.0",
        note: "migration briefing overhaul, structured logging, compat matrix",
    },
    CompatEntry {
        aibox_version: "0.17.7",
        processkit_version: "v0.8.0",
        note: "migration briefing accuracy fixes, version in help header",
    },
    CompatEntry {
        aibox_version: "0.17.8",
        processkit_version: "v0.8.0",
        note: "migration briefing: distinguish sequential vs duplicate migrations",
    },
    CompatEntry {
        aibox_version: "0.17.9",
        processkit_version: "v0.8.0",
        note: "\"latest\" sentinel for aibox and processkit version fields",
    },
    CompatEntry {
        aibox_version: "0.17.10",
        processkit_version: "v0.8.0",
        note: "fix: validate() rejected \"latest\" in [aibox].version (regression from v0.17.9)",
    },
    CompatEntry {
        aibox_version: "0.17.11",
        processkit_version: "v0.8.0",
        note: "fix: [aibox].version = \"latest\" resolved to concrete image tag before Dockerfile generation",
    },
    CompatEntry {
        aibox_version: "0.17.12",
        processkit_version: "v0.8.0",
        note: "yazi git.yazi plugin; Linux/Windows gitignore entries; template-snapshot diff guidance in migration docs",
    },
    CompatEntry {
        aibox_version: "0.17.13",
        processkit_version: "v0.8.0",
        note: "fix: mandatory MCP server enforcement (closes #40); Rust addon linker + x86_64 cross-compile support; Zellij leader Ctrl+g; yazi git status indicators; zellij scratch pad",
    },
    CompatEntry {
        aibox_version: "0.17.14",
        processkit_version: "v0.8.0",
        note: "fix: docs-docusaurus addon installs @docusaurus/core (closes #41); pin default version to 3.8 (closes #42)",
    },
    CompatEntry {
        aibox_version: "0.17.15",
        processkit_version: "v0.13.0",
        note: "fix: gitignore OS patterns + .aibox/; gitignore + scaffold generated MCP client configs; [mcp] section in aibox.toml + .aibox-local.toml; Zellij leader hints via zjstatus; remove dangerous Ctrl+q from normal mode; restore deleted schemas/v1.0.0; docs updated",
    },
    CompatEntry {
        aibox_version: "0.17.16",
        processkit_version: "v0.13.0",
        note: "fix: zellij --layout flag position; Rust x86_64 target added in builder stage; rename ai provider 'codex' → 'openai' (BREAKING: update providers = [\"openai\"] in aibox.toml); add ai-openai addon to install.sh",
    },
    CompatEntry {
        aibox_version: "0.17.17",
        processkit_version: "v0.13.0",
        note: "aibox.toml inline addon documentation; ai-openai addon dep fix",
    },
    CompatEntry {
        aibox_version: "0.17.18",
        processkit_version: "v0.13.0",
        note: "fix: ai-openai addon npm install -g ran as USER aibox causing EACCES; fix: broken ai-codex link in ai-mistral docs",
    },
    CompatEntry {
        aibox_version: "0.17.19",
        processkit_version: "v0.13.0",
        note: "fix: rust addon COPY --from=rust-builder left .cargo/.rustup owned by root; add chown before USER aibox switch",
    },
    CompatEntry {
        aibox_version: "0.17.20",
        processkit_version: "v0.13.0",
        note: "runtime migration id collisions fix; codex auth persistence; preserve .aibox-home via runtime migrations; narrow reset backups; yazi/lazygit theme fixes",
    },
    CompatEntry {
        aibox_version: "0.18.0",
        processkit_version: "v0.13.0",
        note: "harness/provider split ([ai].harnesses + [ai].model_providers); theme auto-apply + WCAG audit; version resolution fixes; backward-compat for legacy [ai].providers",
    },
    CompatEntry {
        aibox_version: "0.18.1",
        processkit_version: "v0.13.0",
        note: "fix: rename ai-openai addon → ai-codex to match AiHarness::Codex addon_name(); add backward compat migration for [addons.ai-openai.tools]",
    },
    CompatEntry {
        aibox_version: "0.18.2",
        processkit_version: "v0.14.0",
        note: "yazi dir preview, git status signs, status bar, scratch pad removal",
    },
    CompatEntry {
        aibox_version: "0.18.3",
        processkit_version: "v0.17.0",
        note: "bump default processkit to v0.17.0; sync baseline-snapshot ordering fix; restore v0.14.0 baseline; 8-role AI-agent team scaffolding",
    },
    CompatEntry {
        aibox_version: "0.18.4",
        processkit_version: "v0.17.0",
        note: "INCOMPLETE RELEASE — tag cut before the multi-version-upgrade fixes landed; Cargo.toml was also not bumped so the shipped binary self-reports as 0.18.3. Skip this version; use 0.18.5 or later.",
    },
    CompatEntry {
        aibox_version: "0.18.5",
        processkit_version: "v0.18.1",
        note: "catch-up release: completes the 0.18.4 work (multi-version upgrade gaps closed); bumps default processkit to v0.18.1 (hookEventName hotfix + src↔context parity); hook commands now use $CLAUDE_PROJECT_DIR so they work regardless of Claude Code's launch cwd; maintain.sh release now writes Cargo.toml + refreshes Cargo.lock before tagging.",
    },
    CompatEntry {
        aibox_version: "0.18.6",
        processkit_version: "v0.18.1",
        note: "MCP-merge release: fixes the flat one-level walker bug in mcp_registration.rs and claude_commands.rs that prevented .mcp.json and .claude/commands/ from being populated against the category-nested skills tree (aibox#53); promotes skill-gate to MANDATORY_MCP_SKILLS so acknowledge_contract() is reachable on every harness session and the PreToolUse compliance gate is satisfiable out of the box; adds collision guard for duplicate skill basenames across categories; repairs cmd_docs_deploy (gh-pages worktree git identity + tmpdir unbound trap).",
    },
    CompatEntry {
        aibox_version: "0.18.7",
        processkit_version: "v0.18.2",
        note: "MCP safety + ergonomics release: hard-fail safety rail in mcp_registration.rs validates every merged MCP script path exists on disk (caught the 12 stale processkit-side mcp-config.json paths reported as processkit#8, fixed upstream in processkit v0.18.2); compliance contract drift checker now tolerant of v1 OR v2 markers (Option C — bridges the transitional state where AGENTS.md template ships v2 but skill-gate's contract source is still v1); devcontainer drift fix — generated file headers no longer stamp the live CLI version, and aibox.lock preserves prior synced_at / installed_at when nothing else changed (clean container rebuild is now a true no-op for git status); gh-pages auto-config probe-first (eliminates the spurious 'Could not configure Pages automatically' warning on every release when Pages is already managed); ships .opencode/plugins/processkit-gate.ts to enforce the compliance contract on OpenCode sessions (closes aibox#51, requires upstream sst/opencode#2319 and #5894 — both shipped).",
    },
    CompatEntry {
        aibox_version: "0.19.0",
        processkit_version: "v0.21.0",
        note: "Minor release: integrates processkit v0.21.0 (major upstream update with enhanced content structure); ships global MCP permission configuration across 8 harnesses (Claude Code, OpenCode, Continue, Cursor, Gemini, Copilot, Aider, Codex) via [mcp.permissions] in aibox.toml with glob pattern matching and deny-precedence semantics; completed backlog grooming with 90-day focus established; all tests passing (597 unit + 41 E2E + 16 integration).",
    },
    CompatEntry {
        aibox_version: "0.19.1",
        processkit_version: "v0.21.0",
        note: "Patch release: applies processkit v0.21.0 migration (564 new files: skills, schemas, roles, bindings, models); updates aibox runtime templates (27 new .aibox-home/* files); all 654 tests passing (unit + integration + E2E).",
    },
    CompatEntry {
        aibox_version: "0.19.2",
        processkit_version: "v0.21.0",
        note: "Patch release: implements MCP config fingerprint tracking (issue #54) to detect per-skill config drift without version bumps; all 660 tests passing.",
    },
    CompatEntry {
        aibox_version: "0.20.0",
        processkit_version: "v0.22.0",
        note: "Minor release: install integrity check + self-heal (`aibox doctor --integrity`, `context/.processkit-provenance.toml`, `SyncDecision::Reinstall`); preauth merge into committed `.claude/settings.json` (closes aibox#55, consumes processkit v0.22.0 `skill-gate/assets/preauth.json`); `--no-container` scaffold mode + `AIBOX_NO_CONTAINER` env var on init/sync; new Tier 1 E2E harness (`no_container_harness`, `preauth_merge`); slash-command name collisions hard-fail; content-diff classifies upstream-removed-stale skills; `mcp_config_hash` renamed `processkit_install_hash` and broadened to cover skills/schemas/processes/state-machines; permissions JSON shape bug (`permissions.allow` flat-key) fixed with nested merge preserving user entries; `aibox.toml` template ships commented `[mcp.permissions]` advisory; tolerates upstream PROVENANCE.toml stamping bug in v0.22.0 via tracing::warn!; all 709 tests passing.",
    },
    CompatEntry {
        aibox_version: "0.21.0",
        processkit_version: "v0.22.0",
        note: "Minor release: sync content-diff data-loss fix (closes aibox#57) — RemovedUpstreamStale reclassification now requires live SHA == older-mirror SHA, preserving user-extended files; same-version sync short-circuit (closes aibox#56) — run_content_sync returns empty diff when from_pk.version == config.processkit.version, no migration document written; multi-harness slash-command scaffolding via new `harness_commands` engine — Codex (`.aibox-home/.codex/prompts/`), Cursor (`.cursor/commands/`), Gemini (`.gemini/commands/` with TOML conversion), OpenCode (`.opencode/commands/`), all gated on `config.ai.harnesses`; `claude_commands.rs` removed and merged into the generic engine; doctor command-registration check now per-harness; sync_perimeter extended; all 654 tests passing.",
    },
    CompatEntry {
        aibox_version: "0.21.1",
        processkit_version: "v0.23.0",
        note: "Patch release: integrates processkit v0.23.0 (RoyalFern governance fields on 34 model entries, new `release-audit` skill, `pk-doctor` `skill_dag` check, `skill-finder` `catalog()` MCP tool, prefix-based PreToolUse matcher in `skill-gate`, YAML literal-block-scalar serialization in `_lib/processkit/frontmatter.py`, `event-log` actor-field validation, content-only layer bumps on 4 skills); applies the resulting migration (50 mirror updates + 9 new upstream files); bumps PROCESSKIT_DEFAULT_VERSION to v0.23.0; no aibox CLI behavior changes — purely a content integration pass.",
    },
    CompatEntry {
        aibox_version: "0.21.2",
        processkit_version: "v0.23.1",
        note: "Patch release: integrates processkit v0.23.1 (release-audit cleanup — 106 ERROR → 0 ERROR via three documented validator relaxations: skip aibox CLI migration prose under `context/migrations/`, model the team-member directory layout / `Persona` kind, make `metadata.processkit.layer` optional for non-processkit-category skills; backfills `metadata.processkit.layer` on 7 processkit-category SKILL.md files; authors missing `## Overview` / `## Full reference` sections on team-creator and team-manager; +24 new pytest cases). FORMAT.md unchanged — no vocabulary updates needed; bumps PROCESSKIT_DEFAULT_VERSION to v0.23.1; no aibox CLI behavior changes — purely a content integration pass.",
    },
    CompatEntry {
        aibox_version: "0.22.0",
        processkit_version: "v0.24.0",
        note: "Minor release: integrates processkit v0.24.0 (context-archiving skill and MCP server, archive-aware index metadata, richer model/provider characteristics, class-based model routing, task-router v0.2 semantic scoring, context-consumption reports, team/model assignment bindings, optional sqlite-vec semantic search, and library-expert skill template). FORMAT.md unchanged — no vocabulary updates needed; bumps PROCESSKIT_DEFAULT_VERSION to v0.24.0.",
    },
    CompatEntry {
        aibox_version: "0.23.0",
        processkit_version: "v0.25.0",
        note: "Minor release: integrates processkit v0.25.0 and its processkit-gateway, collapses processkit MCP registration to a provider-neutral gateway by default, adds daemon-proxy mode, restores apply --no-cache, fixes selected-harness layout generation, adds runtime resource diagnostics and Compose init reaping for Codex sandbox helpers, moves gh/lazygit and voice/preview helpers into selectable addons, adds profile-aware addon metadata and environment-contract projections, and refreshes the docs for runtime operations and processkit v0.25.0.",
    },
    CompatEntry {
        aibox_version: "0.23.1",
        processkit_version: "v0.25.0",
        note: "Patch release: fixes stale runtime cleanup when old same-name containers belong to a previous Compose project, refuses to attach to mismatched Compose-project containers, honors explicit addon tool disablement for lazygit, and adds procps to the base image for runtime process diagnostics.",
    },
    CompatEntry {
        aibox_version: "0.23.2",
        processkit_version: "v0.25.1",
        note: "Patch release: integrates processkit v0.25.1 model-recommender updates: lifecycle metadata for model roster entries, 42 task suitability classes, task-class-aware query_models/resolve_model routing, list_task_classes(), and refreshed current-market model entries. FORMAT.md unchanged; processkit vocabulary unchanged apart from PROCESSKIT_DEFAULT_VERSION.",
    },
    CompatEntry {
        aibox_version: "0.23.3",
        processkit_version: "v0.25.3",
        note: "Patch release: integrates processkit v0.25.3 model-spec/model-profile migrations and makes the Codex seccomp=unconfined bubblewrap fallback part of generated docker-compose.yml for downstream projects.",
    },
    CompatEntry {
        aibox_version: "0.23.4",
        processkit_version: "v0.25.4",
        note: "Patch release: integrates processkit v0.25.4 gateway stdio-proxy daemon startup fixes, wires Codex pre_tool_use hook generation for processkit compliance gating, adds [customization.zellij_status] presentation control, and repairs stale managed Zellij status layouts during runtime sync.",
    },
    CompatEntry {
        aibox_version: "0.23.5",
        processkit_version: "v0.25.4",
        note: "Patch release: fixes generated Dockerfile lazygit disablement cleanup so a missing apt package no longer aborts `aibox apply --no-cache`, while still removing lazygit binaries inherited from older base images.",
    },
    CompatEntry {
        aibox_version: "0.23.6",
        processkit_version: "v0.25.5",
        note: "Patch release: integrates processkit v0.25.5 active interlocutor runtime binding reporting, conservative Claude subagent model emission, and explicit subagent MCP lifecycle guardrails; fixes Codex subagent MCP script paths, addon dependency fallback migrations, doctor aibox.toml/runtime-theme diagnostics, lazygit-disabled runtime cleanup, and native Zellij status-line visibility; strengthens no-container and asciinema E2E coverage.",
    },
    CompatEntry {
        aibox_version: "0.23.7",
        processkit_version: "v0.25.5",
        note: "Patch release: fixes host-generated Codex processkit-gateway MCP paths for devcontainer runtimes, keeping subagent-safe absolute paths while targeting the container workspace mount; doctor now warns about stale host-side Codex MCP script paths.",
    },
    CompatEntry {
        aibox_version: "0.23.8",
        processkit_version: "v0.25.5",
        note: "Patch release: fixes the native aibox Zellij status plugin artifact so Zellij can load its literal WASM entrypoints, restores readable theme-default foreground rendering for the key/status rows, and adds no-container E2E coverage for the export and foreground regressions.",
    },
    CompatEntry {
        aibox_version: "0.23.9",
        processkit_version: "v0.25.6",
        note: "Patch release: restores the shell-backed Zellij status rows as the default, hardens aibox-status against /proc races, fixes Yazi's edit action by using Zellij's editor action instead of injecting Vim commands, applies addon dependency fallback handling to `aibox up`, and integrates processkit v0.25.6 provider-neutral pk command projections.",
    },
    CompatEntry {
        aibox_version: "0.23.10",
        processkit_version: "v0.25.7",
        note: "Patch release: integrates processkit v0.25.7 model-routing content, migrates aibox.toml toward the current apiVersion/kind/metadata/image structure during apply, keeps generated config comments self-documenting, exposes image-slimming tool switches through addons, and finalizes generated-runtime release handling.",
    },
    CompatEntry {
        aibox_version: "0.23.11",
        processkit_version: "v0.25.7",
        note: "Patch release: groups generated aibox.toml around aibox, container, processkit, and ai ownership boundaries; adds catalog-style AI harness and model-provider controls; exposes generated path settings; defaults new projects to the product skill set; and repairs managed Zellij status runtime files during sync.",
    },
    CompatEntry {
        aibox_version: "0.23.12",
        processkit_version: "v0.25.8",
        note: "Patch release: integrates processkit v0.25.8 Xiaomi MiMo model-routing content and cleanup-hint provenance; adds native aibox Zellij key/status bar refinements; moves AI harness and audio install controls under their semantic config sections; fixes Claude CLI installation to a stable /usr/local/bin path; rejects misplaced addon tool entries with owner hints; and warns on stale processkit-managed skills left hot after upstream renames.",
    },
    CompatEntry {
        aibox_version: "0.23.13",
        processkit_version: "v0.25.8",
        note: "Patch release: fixes 0.23.11-to-0.23.12 upgrades for generated configs that still place a moved tool under its old addon owner; apply now migrates misplaced addon tool entries to their unique current catalog owner before strict validation and comment refresh, while preserving hard errors for unknown tools.",
    },
    CompatEntry {
        aibox_version: "0.23.14",
        processkit_version: "v0.25.8",
        note: "Patch release: makes per-harness [ai.harness.<name>] tables the canonical generated aibox.toml selector, adds opt-in `aibox apply --standardize-config` for schema-clean canonical rewrites, removes stale/deprecated generated comments, restores Yazi `e` cross-pane editor handoff, and refreshes docs around semantic AI harness configuration.",
    },
    CompatEntry {
        aibox_version: "0.23.15",
        processkit_version: "v0.25.8",
        note: "Patch release: fixes a 0.23.14 standardize-config regression where a blank [ai.harness.<name>] table could re-enable a harness after its controls were commented out, and restores the standard processkit skill list when canonical config rewrites encounter an empty skill include list.",
    },
    CompatEntry {
        aibox_version: "0.23.16",
        processkit_version: "v0.25.8",
        note: "Patch release: moves Claude processkit command shims to Claude Code's current Skills layout, cleans legacy managed .claude/commands files, fixes the native Zellij key-hint row render trigger, keeps Vim editor panes hot so Yazi edit handoff reaches Vim instead of bash, and adds a pre-release dependency/harness state report.",
    },
    CompatEntry {
        aibox_version: "0.23.17",
        processkit_version: "v0.25.8",
        note: "Patch release: installs Claude Code from Anthropic's signed apt repository with a stable /usr/local/bin/claude path, makes the native Zellij status/key-hint plugin the generated default, starts shell and lazygit tabs hot across layouts, bumps Zellij/Yazi/uv/Cargo dependencies, and improves release-state dependency and harness reporting with harness version-pin support.",
    },
    CompatEntry {
        aibox_version: "0.23.18",
        processkit_version: "v0.25.8",
        note: "Patch release: updates generated Yazi config and theme filetype rules for Yazi 26's url/mime matcher schema, provides writable XDG state mounts for lazygit and similar TUIs, and records follow-up backlog items for native Zellij plugin runtime diagnostics and host-phase runtime smoke tests.",
    },
    CompatEntry {
        aibox_version: "0.23.19",
        processkit_version: "v0.25.8",
        note: "Patch release: hardens generated runtime startup by keeping Vim eager while disabling its startup cursor-position probe, removes suspended generated AI panes, pre-seeds native Zellij plugin permissions, fixes service-specific Codex bubblewrap seccomp fallback, updates Yazi git/preview config, and adds generated-runtime plus opt-in visual E2E release gates.",
    },
    CompatEntry {
        aibox_version: "0.23.20",
        processkit_version: "v0.25.8",
        note: "Patch release: makes the release runtime smoke harness host-safe by defaulting to the shell Zellij status mode, capturing raw TUI output into logs instead of streaming escape sequences to the host terminal, and asserting on structured probe markers rather than terminal transcripts.",
    },
    CompatEntry {
        aibox_version: "0.23.21",
        processkit_version: "v0.25.8",
        note: "Patch release: repairs generated Yazi git/status initialization for Yazi 26, preserves native Zellij plugin permission caches across runtime starts, adds doctor and E2E guardrails for native Zellij permission-cache projection drift, installs the Yazi `ya` companion entrypoint in runtime images, and slims visual E2E release gates with per-case progress logging plus an opt-in exhaustive matrix.",
    },
    CompatEntry {
        aibox_version: "0.24.0",
        processkit_version: "v0.25.8",
        note: "Minor release: adds the bounded diagnostics sidecar, replaces the shell fan-out aibox-status helper with Rust snapshot readers, wires sidecar-backed Zellij status rows, adds `aibox emergency <harness>` recovery startup, keeps legacy native/hidden status aliases while emitting sidecar/disabled, and reduces host release smoke to a minimal default tier with opt-in addon/full tiers.",
    },
    CompatEntry {
        aibox_version: "0.24.1",
        processkit_version: "v0.25.8",
        note: "Patch release: fixes generated compose so the main service starts with the image default root entrypoint user again, allowing entrypoint.sh to remap/drop to aibox instead of failing with `failed switching to \"aibox\": operation not permitted` during release-host runtime smoke.",
    },
    CompatEntry {
        aibox_version: "0.24.2",
        processkit_version: "v0.25.8",
        note: "Patch release: stabilizes Zellij session refresh by preserving running sessions on plain `aibox up`, adding `aibox up --forget-zellij-state`, making apply/fresh starts recreate managed layouts, cleaning stale Zellij cache on apply, canceling accidental leader g/G prefix mode, and starting the diagnostics sidecar with compose --no-deps.",
    },
    CompatEntry {
        aibox_version: "0.24.3",
        processkit_version: "v0.25.8",
        note: "Patch release: contains Zellij status CPU load by defaulting generated layouts back to the shell status path, makes aibox-status read current-container metrics directly, and repairs Claude Code derived runtime drift by pruning stale processkit MCP permissions, seeding a stable claude shim, and reporting drift in doctor.",
    },
    CompatEntry {
        aibox_version: "0.25.0",
        processkit_version: "v0.25.8",
        note: "Major release: removes the Zellij runtime stack and replaces it with tmux-native layouts, tmux attach/session management, host-persistent tmux configuration, preinstalled pinned aibox-managed tmux plugins, TPM as a user convenience layer only, and optional tmux-resurrect/tmux-continuum installed but disabled by default while persistence policy is finalized.",
    },
    CompatEntry {
        aibox_version: "0.25.1",
        processkit_version: "v0.25.8",
        note: "Patch release: keeps the aibox runtime diagnostics segment visible when tmux powerline status is enabled by injecting an aibox PowerKit plugin into the pinned runtime image and tightening release-host smoke coverage for the PowerKit-rendered status bar.",
    },
    CompatEntry {
        aibox_version: "0.25.2",
        processkit_version: "v0.25.8",
        note: "Patch release: adds tmux-native `prefix ?` keybinding help popup, upgrades the two-line PowerKit status row with pane context and tmux mode detail, labels status metrics in aibox-status (CPU/LOAD/NET/MEM/DISK/LOG/OOM/PROC/AI/MCP/MIG/UP), fixes startup regression by targeting named tmux windows instead of `$session:1`, extends release/runtime caching (buildx registry cache refs, E2E Dockerfile dependency split, no podman-compose in runner), and documents provider endpoint base URL hints (ANTHROPIC_BASE_URL/OPENAI_BASE_URL/GEMINI_BASE_URL/MISTRAL_BASE_URL).",
    },
    CompatEntry {
        aibox_version: "0.25.3",
        processkit_version: "v0.25.8",
        note: "Patch release: fixes Yazi `e` editor handoff for tmux by targeting the existing Vim pane/window without spawning nested panes, returns focus to Yazi on `:q`, enables Vim mouse/no-wrap scrolling defaults, documents tmux status modes and element toggles, makes skill-finder discover deselected skills from the template catalog and lazy-install matches, adds SSH-first E2E companion guidance with uidmap preflight, and tightens host runtime smoke/persistence guardrails.",
    },
    CompatEntry {
        aibox_version: "0.25.4",
        processkit_version: "v0.25.8",
        note: "Patch release: repairs tmux release-host smoke probing after the managed tmux socket migration and fixes generated tmux status-right rendering by preserving the aibox runtime status segment.",
    },
    CompatEntry {
        aibox_version: "0.25.5",
        processkit_version: "v0.25.8",
        note: "Patch release: refreshes managed tmux runtime files when recreating sessions so aibox.toml status/layout settings take effect, preserves the delayed Yazi pane startup path, and suppresses stale default-socket tmux kill-session noise on host attach.",
    },
    CompatEntry {
        aibox_version: "0.25.6",
        processkit_version: "v0.25.8",
        note: "Minor release (v0.25.6 host-orchestration rollout): \
               cross-version sync auto-recovers corrupted managed runtime files (off_RIGHT fix, commit e0ee7bc); \
               generic purge-on-disable for all addon tools — kubernetes, cloud-aws/azure/gcp, infrastructure, audio-voice, preview-archive, preview-enhanced, data-preview, yazi-omp, and existing git-ui pattern; \
               new [apply].purge_disabled_harness_state toml key (default false); \
               BREAKING: [customization.zellij_status] is now schema-rejected — remove from aibox.toml before upgrade (see docs-site/docs/migrations/zellij-eol.md); \
               six new doctor checks + semver-aware version-skew reporting; \
               addon download integrity hardening — 11 addons use SHA-256/GPG/.sha256 sidecar verification; ai-hermes and ai-opencode pinned to GitHub release assets (vendors publish no SHA256SUMS — see TODO annotations); \
               BREAKING: seccomp=unconfined now requires [security].acknowledge_seccomp_unconfined = true in aibox.toml; aibox init --harness codex auto-sets this; existing Codex projects must add it manually (see docs-site/docs/reference/security.md); \
               aibox.toml [skills] dedup — single array of strings, comment-out to disable; \
               two-line powerline status bar with six chevron-styled aibox metrics segments (slot order fixed per DEC-20260508_2115-SilentFern); \
               internal: cli/src/seed.rs split into cli/src/tmux/ module (3,613 → 2,929 lines); \
               log pane via lnav + vim hard-cut (Yazi 'e' opens full-screen tmux popup, no persistent vim pane).",
    },
    CompatEntry {
        aibox_version: "0.25.7",
        processkit_version: "v0.26.0",
        note: "Minor release: processkit v0.26.0 integration (RoleSlot primitive, query_budget_drift, route_task response fields, compliance contract v2 rewrite, slim per-turn hook, lazy-import for aggregate-mcp); \
               new McpGatewayMode::LazyAggregate (PROCESSKIT_MCP_MODE=lazy_catalog) opt-in; \
               aibox v1→v2 Migration emission mechanism (cli/src/v1_v2_migration.rs); \
               Phase 0 release ritual now invokes pk-doctor + aibox doctor (AIBOX_RELEASE_SKIP_DOCTORS=1 escape hatch); \
               tmux session name derived from project name (was hardcoded 'aibox'); \
               powerkit four-section statusline (line1-right + line2-left + line2-right per owner spec; paired MIG-STATUSLINE migration); \
               per-layout multi-harness behaviour (browse/cowork/cowork-swap/dev/focus per DEC-TrueClover); \
               tools-as-windows generalization (lazygit window, prefix g/s bindings, framework for future tool addons); \
               terminal-emulator-agnostic env passthrough (kitty/wezterm/iterm2/ghostty/etc.) — fixes yazi RT timeout flash; \
               powerkit plugins de-doubled (OOM/LOG/PROC/AI/MCP/MIG); \
               docs-addons run project-local npm install (prism-react-renderer surprise fix); \
               release-audit stale-test grep sweep; \
               wasm-bindgen + js-sys + cc + filetime + hashbrown lockfile patch updates; \
               uv image bumped 0.11.10 → 0.11.11; Docusaurus pin bumped 3.8 → 3.10.1; \
               Codex startup latency fix (McpGatewayMode::Aggregate eliminates N-process MCP handshakes); \
               aibox sync per-skill mcp-config drift detection (closes #54); \
               aibox v1→v2 Migration emission mechanism (closes #72); \
               release-script ordering: push main before tag, notes-curation checkpoint before gh release create (closes #73 + improves audit trail).",
    },
];

/// Find the minimum compatible processkit version for the given aibox version.
/// Returns `None` if the aibox version is older than any entry in the table.
pub fn min_processkit_for(aibox_version: &str) -> Option<&'static CompatEntry> {
    // Find the entry with the highest aibox_version that is <= aibox_version.
    // Versions are semver strings — parse them for comparison.
    let target = parse_semver(aibox_version)?;

    COMPAT_TABLE.iter().rfind(|e| {
        parse_semver(e.aibox_version)
            .map(|v| v <= target)
            .unwrap_or(false)
    })
}

/// Parse a semver string like "0.17.5" or "v0.17.5" into (major, minor, patch).
fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim_start_matches('v');
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Check if a processkit version string meets the minimum requirement.
/// Both strings should be like "v0.8.0" or "0.8.0".
pub fn processkit_meets_minimum(installed: &str, minimum: &str) -> bool {
    match (parse_semver(installed), parse_semver(minimum)) {
        (Some(inst), Some(min)) => inst >= min,
        _ => true, // if we can't parse, don't warn
    }
}

/// Return the slice of `COMPAT_TABLE` entries whose `aibox_version` is
/// strictly greater than `from_excl` and less than or equal to `to_incl`.
/// Used by the migration document generator to enumerate every released
/// intermediate when a project jumps across multiple CLI versions.
///
/// If either bound fails to parse as semver, falls back to `&[]` (callers
/// downgrade to the generic target-only rendering).
pub fn entries_in_range(from_excl: &str, to_incl: &str) -> Vec<&'static CompatEntry> {
    let (Some(from_v), Some(to_v)) = (parse_semver(from_excl), parse_semver(to_incl)) else {
        return Vec::new();
    };
    if from_v >= to_v {
        return Vec::new();
    }
    COMPAT_TABLE
        .iter()
        .filter(|e| match parse_semver(e.aibox_version) {
            Some(v) => v > from_v && v <= to_v,
            None => false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Release hygiene: every time the CLI version bumps, a corresponding
    /// `COMPAT_TABLE` entry must be added. This test fails the release if
    /// the table is out of date, so the omission is caught before ship.
    #[test]
    fn cargo_pkg_version_has_compat_entry() {
        let cargo = env!("CARGO_PKG_VERSION");
        let found = COMPAT_TABLE.iter().any(|e| e.aibox_version == cargo);
        assert!(
            found,
            "CARGO_PKG_VERSION = {cargo} has no entry in COMPAT_TABLE              (cli/src/compat.rs) — add one alongside the version bump"
        );
    }

    #[test]
    fn entries_in_range_basic() {
        let got: Vec<&str> = entries_in_range("0.17.9", "0.17.12")
            .iter()
            .map(|e| e.aibox_version)
            .collect();
        assert_eq!(got, vec!["0.17.10", "0.17.11", "0.17.12"]);
    }

    #[test]
    fn entries_in_range_cross_minor() {
        let got: Vec<&str> = entries_in_range("0.17.20", "0.18.2")
            .iter()
            .map(|e| e.aibox_version)
            .collect();
        // 0.18.0, 0.18.1, 0.18.2 all must appear.
        assert_eq!(got, vec!["0.18.0", "0.18.1", "0.18.2"]);
    }

    #[test]
    fn entries_in_range_same_version_is_empty() {
        assert!(entries_in_range("0.18.3", "0.18.3").is_empty());
    }

    #[test]
    fn entries_in_range_descending_is_empty() {
        assert!(entries_in_range("0.18.3", "0.17.10").is_empty());
    }

    #[test]
    fn entries_in_range_bad_input_is_empty() {
        assert!(entries_in_range("bogus", "0.18.3").is_empty());
    }
}
