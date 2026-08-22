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
               BREAKING: [customization.zellij_status] is now schema-rejected — remove from aibox.toml before upgrade (see docs-site/content/docs/migrations/zellij-eol.md); \
               six new doctor checks + semver-aware version-skew reporting; \
               addon download integrity hardening — 11 addons use SHA-256/GPG/.sha256 sidecar verification; ai-opencode pinned to GitHub release assets (vendor publishes no SHA256SUMS — see TODO annotations); \
               BREAKING: seccomp=unconfined now requires [security].acknowledge_seccomp_unconfined = true in aibox.toml; aibox init --harness codex auto-sets this; existing Codex projects must add it manually (see docs-site/content/docs/reference/security.md); \
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
    CompatEntry {
        aibox_version: "0.25.8",
        processkit_version: "v0.26.0",
        note: "Patch release: improves the tmux log viewer with mouse scrolling, latest-log positioning, visual session separators, and mixed CLI/runtime event rendering; filters aibox log counts to the current container session using a stable container id; emits low-volume runtime lifecycle samples from the diagnostics sidecar; moves PowerKit status metrics into the owner-specified two-row layout; adds container-uptime reporting and compact MCP mode labels; updates tmux layouts around ordered harness semantics; enables Vim mouse scrolling with nowrap defaults; refreshes preview tool grouping; updates clap_complete in Cargo.lock; and tracks the deferred uv base image pin review.",
    },
    CompatEntry {
        aibox_version: "0.25.9",
        processkit_version: "v0.26.1",
        note: "Patch release: integrates processkit v0.26.1 with pk-doctor entity-storage hygiene coverage and migration/index-management test additions; retires yazi-omp runtime support; migrates tmux status configuration to list-based slot ordering; fixes preview-enhanced addon dependency/config standardization; preserves user-selected themes during standardization; suppresses Kubernetes/cloud PowerKit auth/probe flashes; distinguishes direct vs inherited git status in Yazi directory previews; removes superseded legacy context/models entries after Artifact model-spec migration; and routes uv to a writable /tmp/aibox cache for sandbox-safe first-run behavior.",
    },
    CompatEntry {
        aibox_version: "0.25.10",
        processkit_version: "v0.26.2",
        note: "Patch release: integrates processkit v0.26.2; adds configurable tmux status labels/layouts and model-provider health segments; reduces PowerKit refresh churn; fixes log-viewer Vim yanking through the tmux/host clipboard bridge; stabilizes runtime MCP diagnostics; verifies release checksum sidecars; and improves image layer/cache reuse across apply/up and host release publishing.",
    },
    CompatEntry {
        aibox_version: "0.25.11",
        processkit_version: "v0.26.2",
        note: "Patch release: keeps processkit v0.26.2 and fixes source-checkout addon discovery, PowerKit status cache writability, generated runtime writability diagnostics, disabled-harness migration schema metadata, and tmux/Yazi status glyph defaults.",
    },
    CompatEntry {
        aibox_version: "0.25.12",
        processkit_version: "v0.26.2",
        note: "Patch release: keeps processkit v0.26.2 and fixes fresh-project tmux/PowerKit runtime projection by using broad writable runtime-home mounts, refreshing managed theme/Yazi/Codex files on apply, scoping preauth/MCP writes to enabled harnesses, recognizing nested processkit skill catalogs in doctor, and reducing host-side doctor probes for container-owned dependencies.",
    },
    CompatEntry {
        aibox_version: "0.25.13",
        processkit_version: "v0.26.2",
        note: "Patch release: keeps processkit v0.26.2 and fixes stale runtime-home propagation by making aibox-managed .aibox-home files authoritative on apply and clean runtime recreation; broadens generated runtime mounts for Vim and Cargo cache directories; updates generated compose/docs coverage; and adds regression coverage for stale tmux/Yazi managed file refresh.",
    },
    CompatEntry {
        aibox_version: "0.25.14",
        processkit_version: "v0.26.5",
        note: "Patch release: integrates processkit v0.26.5; restores generated and image fallback Alt-word movement in Vim/readline; adds managed .inputrc runtime projection; carries tmux clipboard/terminal feature improvements into generated and image fallback configs; keeps Rust cache mounts from shadowing image-provided cargo/rustc shims; and keeps the stricter processkit schema/file-layout migration path clean under pk-doctor.",
    },
    CompatEntry {
        aibox_version: "0.26.0",
        processkit_version: "v0.26.7",
        note: "Minor release: theme palette overhaul (Rose Pine Moon / GitHub Light / Ayu Light / Projectious fixes, AI TUI sync for Claude/Aider/Gemini/OpenCode, custom PowerKit theme so chevrons land on the active surface bg, full palette-driven Yazi/Vim, inactive-pane dim style); live Prefix+L layout chooser and Prefix+T theme chooser with confirm-dialog enumerating impacted apps; Yazi rich-preview position indicator and aibox-preview rich pipeline matching the in-pane renderer; Vim Alt-arrow word jumps now stay in INSERT (ttimeoutlen + per-letter terminal-keycode registration) plus Shift-Alt-arrow visual selection; model-provider chevrons drop the redundant ✓ glyph and add per-provider agent count via aibox-status ai_agents_breakdown (opt-in quota / admin-usage polling); bat/delta/fzf/eza/lnav/less wired through theme-env.sh; Tier 3 vt100 rendered-color e2e suite (Starship/tmux/Yazi/layout-switch/theme-switch) catches the historical 'tmux bg rendered black' and 'status line 1 silently empty' regression classes; processkit bumped to v0.26.7.",
    },
    CompatEntry {
        aibox_version: "0.26.1",
        processkit_version: "v0.26.9",
        note: "Patch release: fixes the v0.26.0 GHCR push silent-failure (release-host now verifies every <flavor>-vX.Y.Z and <flavor>-latest tag via buildx imagetools inspect post-publish, dying loud on missing tags or versioned/latest digest mismatch); fixes yazi rich-preview Lua error 'attempt to call a nil value (field preview_widgets)' by renaming to ya.preview_widget (singular, yazi 26.x API) and adds a cache-backed scrolling layer keyed by (path, mtime, width) under ${XDG_CACHE_HOME:-~/.cache}/aibox-yazi-rich-preview/ so Alt-J/Alt-K no longer re-spawns Python on every keystroke; fixes the recurring 'Repairing processkit template mirror — install_hash_mismatch' warn that fired on every `aibox apply` because legitimate agent edits to per-skill <skill>/config/settings.toml flipped the integrity fingerprint — the hash now skips agent-mutable config/ dirs and tags its output with a v2: format prefix so legacy untagged hashes upgrade silently; emits spec.affected_files in Migration entity frontmatter for content_diff and runtime_sync writers (projectious-work/aibox#74) so pk-doctor's migration_integrity.affected-files-empty stays clean; integrates processkit v0.26.7→v0.26.9 (id-vocabulary check now walks configured per-kind palettes with 50k+ target instead of measuring the global default pool, runtime-prune included in preauth.json).",
    },
    CompatEntry {
        aibox_version: "0.26.2",
        processkit_version: "v0.26.9",
        note: "Patch release: fixes the user-impacting 'duplicate key labels in table customization.tmux.status' parse failure on aibox apply — migrate_aibox_toml_structure now pre-normalizes duplicate single-bracket table headers before strict toml_edit parsing (last-write-wins on colliding keys, array-of-tables left alone, comments preserved from the first occurrence); patches the three aibox-tmux helper scripts (switch-layout, confirm-and-switch, refresh-theme) to thread AIBOX_TMUX_SOCKET through their own tmux invocations via a tmux() function wrapper so users on non-default sockets no longer see operations land on /tmp/tmux-$UID/default; removes the structurally-broken Tier 3 visual_rendered_tmux.rs + visual_rendered_yazi.rs e2e files (capture-pane cannot see tmux status bar, so the assertions could never fire) — the asciinema-based visual.rs and visual_matrix.rs suites cover the same regression classes correctly and pass; live layout-switch + theme-switch coverage tracked for an asciinema rewrite in BACK-20260514_1752-EarnestMoss.",
    },
    CompatEntry {
        aibox_version: "0.26.3",
        processkit_version: "v0.26.9",
        note: "Patch release: fixes fetch_latest_image_version (cli/src/update.rs) silently capping at the first GHCR tag-list page so freshly-published images were invisible until enough older tags fell off — `aibox apply` kept resolving `latest` to a long-stale image (e.g. v0.25.12 after v0.26.0–v0.26.2 had already shipped). The resolver now requests `?n=1000` on the first call AND walks Docker Registry v2 `Link: <…>; rel=\"next\"` pagination (capped at 50 pages defensively). 6 new unit tests cover the Link-header parser (relative vs absolute URLs, quoted vs unquoted rel, multi-relation headers, malformed inputs). The release-host verifier from v0.26.1 (verify_release_images_in_ghcr) was unaffected — it uses per-tag buildx imagetools inspect rather than the tag-list endpoint. Tracked as BACK-20260514_1902-ShinyLake.",
    },
    CompatEntry {
        aibox_version: "0.26.4",
        processkit_version: "v0.26.10",
        note: "Patch release: integrates processkit v0.26.10; makes explicit tmux model-provider status elements render even without global provider polling; preserves Claude Code OAuth/state across container rebuilds via Claude XDG cache/config/state mounts; enables terminal extended-key passthrough for Alt-Enter; adds PowerKit GitHub issue/PR counts; and fixes aibox-status AI agent counting so vendor helper processes do not inflate provider instance totals.",
    },
    CompatEntry {
        aibox_version: "0.26.5",
        processkit_version: "v0.26.13",
        note: "Patch release: integrates processkit v0.26.13; re-architects the customization theme model so the user picks a family ('ayu', 'gruvbox', etc.) plus mode (auto|light|dark) and an optional alternate variant — legacy concrete names like 'ayu-dark' still parse and round-trip via standardize-config (DECAY: legacy_theme sidecar locks the resolved palette); integrates 32 new themes from the project owner's curated theme-explorer reference (Monokai, OneDark, Vitesse, Min, Kanagawa, Slack, Everforest, VsCode, plus solo families Andromeeda, AuroraX, Houston, Laserwave, Plastic, Poimandres, Red, Snazzy, Synthwave84, Vesper) wired into every theme renderer (palette, vim, tmux/powerkit, yazi, lazygit, starship, fzf, eza, bat, delta, lnav, claude/codex/aider/gemini/opencode); upgrades scripts/test-screencasts.sh with a Python ANSI cast-invariants helper that catches powerkit segment-rendering bugs (caught the 'GH preceded by two black arrows' regression) and a powerkit-aware demo driver; ships per-theme demo recordings under docs-site/static/asciinema/themes/ + a Docusaurus theme-gallery page; shrinks the release-gated visual_matrix.rs companion suite from 7 themes to 1 (full sweep behind AIBOX_E2E_VISUAL_FULL_MATRIX=1); adds a Dockerfile-baked sed patch for tmux-powerkit's segment_builder.sh (tracked by BACK-20260515_1503-RefinedIvy for upstream); ships a new powerkit 'forge' plugin that auto-detects the git remote provider (github/gitlab/codeberg/forgejo/gitea) and renders branch + I + P in one segment, replacing the stand-alone 'git' + 'github' default and de-duplicating the doubled-branch artifact; redesigns the Leader+? cheatsheet popup with categorized 2-column rows (Panes/Windows/Layouts/Themes/Sessions/Copy/aibox/Vim-tmux); fixes Claude login state being wiped by sync_theme_files force-overwriting .claude.json (added is_bind_mount_stub_file guard); preserves legacy concrete theme intent on standardize-config (deserializer derives mode + variant from the legacy concrete so re-serialization round-trips correctly).",
    },
    CompatEntry {
        aibox_version: "0.26.6",
        processkit_version: "v0.26.14",
        note: "Patch release: integrates processkit v0.26.14; updates processkit metadata to the new release and preserves tmux/Vim/cheatsheet and clipboard behavior consistency.",
    },
    CompatEntry {
        aibox_version: "0.26.7",
        processkit_version: "v0.26.15",
        note: "Patch release: integrates processkit v0.26.15; adds provider-neutral AI execution policy axes for filesystem, approval, and network behavior; adds per-harness execution overrides; maps execution policy to Codex settings generation; preserves the processkit MCP manifest in derived installs; refreshes documentation recordings; and keeps MCP permission configuration focused on tool allow/deny intent.",
    },
    CompatEntry {
        aibox_version: "0.26.8",
        processkit_version: "v0.26.16",
        note: "Patch release: integrates processkit v0.26.16; refreshes the processkit template mirror, provenance, MCP manifest, TeamMember privacy defaults, and team consistency checks; makes latest image resolution skip GHCR tags whose multi-arch manifest children have been pruned.",
    },
    CompatEntry {
        aibox_version: "0.27.0",
        processkit_version: "v0.27.0",
        note: "Minor release: integrates processkit v0.27.0 and the v0.26.17 supply-chain audit surface; switches the next-minor GHCR image scheme to foundation/runtime tags, stops publishing public source-hash marker tags, preserves legacy base-debian-v0.26.x compatibility, adds GHCR source-tag cleanup tooling, adds release LICENSE guardrails, and fixes pasted host newlines by removing global tmux C-j navigation.",
    },
    CompatEntry {
        aibox_version: "0.27.1",
        processkit_version: "v0.27.0",
        note: "Patch release: refreshes addon and user-facing toolchain pins, including documentation generators, language package managers, infrastructure tooling, Kubernetes tools, Helm, and OpenCode; adds release-state coverage for addon pins plus LaTeX and apt-managed inputs; keeps Python interpreter selection tied to the Debian base image package set; fixes OpenCode release asset naming and checksum verification; updates docs-site dependencies and clears npm audit findings.",
    },
    CompatEntry {
        aibox_version: "0.27.2",
        processkit_version: "v0.27.0",
        note: "Patch release: fixes the docs-hugo addon checksum verification so Hugo archives downloaded to /tmp/hugo.tar.gz are checked against the matching release checksum entry instead of the upstream asset filename.",
    },
    CompatEntry {
        aibox_version: "0.27.3",
        processkit_version: "v0.27.1",
        note: "Patch release: integrates processkit v0.27.1 derived-project health cleanup, including quieter pk-doctor sensitive-data checks, gateway-aware preauth validation, sqlite-vec availability for pk-doctor MCP runs, supply-chain policy no-policy INFO handling, and a Codex processkit-gateway startup timeout so uv-backed gateway startup does not trip Codex's 30-second default.",
    },
    CompatEntry {
        aibox_version: "0.27.4",
        processkit_version: "v0.27.1",
        note: "Patch release: refreshes all shipped addon and harness tool defaults to current upstream releases, fixes Yarn 4 and Python 3.14 installation paths, moves Hermes to the current hermes-agent package installer, updates Rust/docs dependencies, and quiets actionable pk-doctor false positives while making release-integrity checks bounded.",
    },
    CompatEntry {
        aibox_version: "0.27.5",
        processkit_version: "v0.27.1",
        note: "Patch release: refreshes the shipped docs-hugo pin to Hugo 0.162.1 and aligns release-state/docs inventory with the current addon and harness defaults from v0.27.4.",
    },
    CompatEntry {
        aibox_version: "0.27.6",
        processkit_version: "v0.27.1",
        note: "Patch release: refreshes tool and harness pins, updates generated runtime/docs references, and adds the GitHub CLI credential-helper integration for HTTPS Git operations.",
    },
    CompatEntry {
        aibox_version: "0.27.7",
        processkit_version: "v0.27.1",
        note: "Patch release: fixes TeX Live's historic installer mirror, resolves pk-doctor false positives, adds disabled harness-state preservation controls, and accepts context mode in schema validation.",
    },
    CompatEntry {
        aibox_version: "0.27.8",
        processkit_version: "v0.27.2",
        note: "Patch release: integrates processkit v0.27.2 derived-project remediation, preserving project-local pk-commands during schema validation, accepting timestamped role-slot bindings, forwarding explicit doctor confirmations, exposing archive remediation metadata, and refreshing gateway metadata.",
    },
    CompatEntry {
        aibox_version: "0.28.0",
        processkit_version: "v0.27.4",
        note: "Minor release: adds named LaTeX build, watch, status, and EmbedPDF live-preview workflows; manages preview startup and teardown with the workspace lifecycle; generates project-local agent guidance; and documents persistent, least-privilege GitHub authentication with explicit per-destination tokens.",
    },
    CompatEntry {
        aibox_version: "0.28.1",
        processkit_version: "v0.27.4",
        note: "Patch release: restores Codex processkit command aliases through /prompts:pk-*; adds local fuzzy documentation search; moves LaTeX build and watch ownership into the development container; and adds a hardened read-only, multi-document preview sidecar with full lifecycle coverage.",
    },
    CompatEntry {
        aibox_version: "0.28.2",
        processkit_version: "v0.27.5",
        note: "Patch release: integrates processkit v0.27.5 derived-project doctor fixes, Git-ignore-aware sensitive-data scanning, archive-age enforcement, generated schema foundations, migration drafting, repository portfolio review, and refreshed gateway metadata.",
    },
    CompatEntry {
        aibox_version: "0.28.3",
        processkit_version: "v0.27.6",
        note: "Patch release: integrates processkit v0.27.6; makes GitHub CLI authoritative for github.com HTTPS credentials when enabled by resetting earlier credential helpers; exposes exact GitHub SSH aliases through the Forge tmux status configuration; and preserves the active page and zoom across LaTeX live-preview rebuilds.",
    },
    CompatEntry {
        aibox_version: "0.28.4",
        processkit_version: "v0.28.1",
        note: "Patch release: integrates processkit v0.28.1 and refreshes the maintained v0.x processkit compatibility baseline.",
    },
    CompatEntry {
        aibox_version: "0.28.5",
        processkit_version: "v0.28.1",
        note: "Patch release: fixes Hermes Agent installation under the non-root runtime model; restores the configured lazygit runtime surfaces; completes processkit reconciliation; and enforces traceable ports between maintained v0.x and v1.x lines.",
    },
    CompatEntry {
        aibox_version: "0.28.6",
        processkit_version: "v0.28.3",
        note: "Patch release: fixes Kubernetes addon checksum verification for Helm, Kustomize, and k9s archives on amd64 and arm64; and integrates processkit v0.28.3 authenticated GitHub repository reconciliation.",
    },
    CompatEntry {
        aibox_version: "0.28.7",
        processkit_version: "v0.28.3",
        note: "Patch release: fixes OpenTofu and Packer addon checksum verification on amd64 and arm64 by preserving upstream archive filenames during checksum validation.",
    },
    CompatEntry {
        aibox_version: "0.28.8",
        processkit_version: "v0.28.3",
        note: "Patch release: makes the infrastructure addon self-sufficient by installing python3-pip before installing Ansible, so generated Dockerfiles build without the Python addon.",
    },
    CompatEntry {
        aibox_version: "0.28.9",
        processkit_version: "v0.28.3",
        note: "Patch release: installs Ansible, Poetry, PDM, and Azure CLI in isolated virtual environments so generated Debian trixie Dockerfiles comply with PEP 668 while keeping their commands available on PATH.",
    },
    CompatEntry {
        aibox_version: "0.28.10",
        processkit_version: "v0.28.3",
        note: "Patch release: reconciles the standard processkit skills, recommends tooling-linked skills interactively, upgrades prerelease processkit surfaces, and serializes release Tier 2 E2E validation for the shared companion.",
    },
    CompatEntry {
        aibox_version: "0.28.11",
        processkit_version: "v0.28.3",
        note: "Patch release: adds the Cloudflare addon, installing cloudflared from Cloudflare's signed repository instead of Debian's archive so generated trixie images build on amd64 and arm64.",
    },
    CompatEntry {
        aibox_version: "0.28.12",
        processkit_version: "v0.28.4",
        note: "Patch release: integrates processkit v0.28.4 and makes companion E2E validation work from linked release worktrees.",
    },
    CompatEntry {
        aibox_version: "0.28.13",
        processkit_version: "v0.28.4",
        note: "Patch release: adds open GitHub Discussion counts to the tmux Forge status segment and restores the complete generated Codex command projection set.",
    },
    CompatEntry {
        aibox_version: "0.28.14",
        processkit_version: "v0.28.4",
        note: "Patch release: ensures pk-reconcile and pk-repo-reconcile install their project-reconciliation and repo-management skill dependencies.",
    },
    CompatEntry {
        aibox_version: "0.28.15",
        processkit_version: "v0.28.4",
        note: "Patch release: refreshes the bundled maintenance tools, locks cargo-audit installation for Rust compatibility, and publishes the Hugo/Docsy documentation site.",
    },
    CompatEntry {
        aibox_version: "0.28.16",
        processkit_version: "v0.28.4",
        note: "Patch release: installs Node.js from checksum-verified official release archives after the NodeSource signing-key endpoint became unavailable and refreshes generated runtime and processkit package-selection state.",
    },
    CompatEntry {
        aibox_version: "0.28.17",
        processkit_version: "v0.28.4",
        note: "Patch release: repairs Go, Typst, AWS CLI, and Node.js add-on installers and adds a clean companion-container build gate for download-based add-on defaults.",
    },
    CompatEntry {
        aibox_version: "0.28.18",
        processkit_version: "v0.28.5",
        note: "Patch release: restores Codex processkit MCP startup by preserving uv run --script in gateway daemon-proxy commands, integrates processkit v0.28.5's MCP 1.x compatibility bound, and restores zero-warning clippy under Rust 1.97.",
    },
    CompatEntry {
        aibox_version: "0.28.19",
        processkit_version: "v0.28.5",
        note: "Patch release: preserves prerelease identifiers when resolving the latest published GHCR image so v1.0.0-alpha.1 is not rewritten to the nonexistent v1.0.0 tag.",
    },
    CompatEntry {
        aibox_version: "0.29.0",
        processkit_version: "v0.28.5",
        note: "Minor release: adds Tau as a first-class multi-provider coding-agent harness with pinned installation, persistent runtime state, AGENTS.md discovery, Agent Skills projection, and explicit MCP capability reporting.",
    },
    CompatEntry {
        aibox_version: "0.30.0",
        processkit_version: "v0.28.5",
        note: "Minor release: adds nested language addon groups, production Go quality tooling, and language-neutral supply-chain and release bundles with pinned versions, checksum verification, and per-tool overrides.",
    },
    CompatEntry {
        aibox_version: "0.30.1",
        processkit_version: "v0.28.5",
        note: "Patch release: refreshes the companion E2E contract, repairs Starship cache isolation, resolves Codex latest pins before container builds, and updates security-relevant pnpm and Tau curated defaults.",
    },
    CompatEntry {
        aibox_version: "0.31.0",
        processkit_version: "v0.28.5",
        note: "Minor release: adds optional rootless Podman and Podman Compose tooling to the infrastructure addon, documents the Go supply-chain and release bundles, and repairs minimal infrastructure addon rendering.",
    },
    CompatEntry {
        aibox_version: "0.31.1",
        processkit_version: "v0.28.6",
        note: "Patch release: repairs incomplete processkit upgrade caches, installs declared skill dependencies, removes stale pk command projections, and consumes source-specific MCP header manifests.",
    },
    CompatEntry {
        aibox_version: "0.31.2",
        processkit_version: "v0.28.6",
        note: "Patch release: replaces privileged companion E2E coverage with isolated local contracts and an owner-controlled, evidence-producing macOS host gate.",
    },
    CompatEntry {
        aibox_version: "0.31.3",
        processkit_version: "v0.28.6",
        note: "Patch release: adds a locked Textual dashboard and reviewed content-addressed cache reuse to the restricted macOS host gate.",
    },
    CompatEntry {
        aibox_version: "0.31.4",
        processkit_version: "v0.28.6",
        note: "Patch release: makes Hugo downloads resilient to transient network failures, improves the release-host Textual problem workflow, and serializes contention-sensitive E2E gates.",
    },
    CompatEntry {
        aibox_version: "0.31.5",
        processkit_version: "v0.28.6",
        note: "Patch release: retries OpenCode release downloads and makes Textual yanks selection-aware while preserving actionable failed-task diagnostics.",
    },
    CompatEntry {
        aibox_version: "0.32.0",
        processkit_version: "v0.28.6",
        note: "Minor release: adds a pinned Chromium-first Playwright and axe browser-testing addon with optional Firefox/WebKit, live release-host browser evidence, and a cleaner full-width Textual release dashboard.",
    },
    CompatEntry {
        aibox_version: "0.32.1",
        processkit_version: "v0.28.6",
        note: "Patch release: makes the browser-testing host gate launch the full Chromium channel installed by Playwright --no-shell instead of requesting the omitted headless-shell executable.",
    },
    CompatEntry {
        aibox_version: "0.32.2",
        processkit_version: "v0.28.6",
        note: "Patch release: uses an explicit Playwright BrowserContext for axe host validation and makes safe release-host caches and candidate-bound retries available by default.",
    },
    CompatEntry {
        aibox_version: "0.32.3",
        processkit_version: "v0.28.6",
        note: "Patch release: makes the axe host fixture accessibility-clean and records structured violation diagnostics when a future browser probe fails.",
    },
    CompatEntry {
        aibox_version: "0.32.4",
        processkit_version: "v0.28.6",
        note: "Patch release: keeps latest image resolution on the active v0 line, refreshes generated addon comments when the catalog changes, and fixes stale or collapsed Yazi Markdown previews.",
    },
    CompatEntry {
        aibox_version: "0.32.5",
        processkit_version: "v0.28.6",
        note: "Patch release: embeds the canonical addon catalog so stale host installs cannot hide shipped tools, refreshes same-version installs, and adds Yazi/Vim clipboard and selectable-preview workflows.",
    },
    CompatEntry {
        aibox_version: "0.32.6",
        processkit_version: "v0.28.6",
        note: "Patch release: refreshes curated tool pins, including Go 1.26.6, and routes every Yazi copy action through the tmux and host clipboard bridge.",
    },
    CompatEntry {
        aibox_version: "0.33.0",
        processkit_version: "v0.28.6",
        note: "Minor release: adds configurable, terminal-neutral tmux titles and lifecycle attention signals for AI harness panes, with optional notifications.",
    },
    CompatEntry {
        aibox_version: "0.33.1",
        processkit_version: "v0.28.6",
        note: "Patch release: restores generated AI harness startup under nounset, keeps Yazi directory previews compatible across supported image pins, and aligns PowerKit plugin spacing with window tabs.",
    },
    CompatEntry {
        aibox_version: "0.33.2",
        processkit_version: "v0.28.8",
        note: "Patch release: updates the default processkit release to v0.28.8, refreshes generated runtime metadata for Codex 0.148.0, and reconciles the completed live tmux layout and theme switching work.",
    },
    CompatEntry {
        aibox_version: "0.34.0",
        processkit_version: "v0.28.8",
        note: "Minor release: adds configurable agent-aware tmux headers and replaces the legacy documentation stack with the projectious.work Hugo brand theme.",
    },
    CompatEntry {
        aibox_version: "0.34.1",
        processkit_version: "v0.28.8",
        note: "Patch release: expands theme palettes across the managed terminal toolchain, adds exact Codex syntax themes, restores the visual theme gallery, and clears Codex question state after permission answers.",
    },
    CompatEntry {
        aibox_version: "0.34.2",
        processkit_version: "v0.28.8",
        note: "Patch release: publishes the generated theme gallery and chooser, aligns terminal themes with the design reference, reliably clears answered Codex question state, and improves active tmux pane visibility.",
    },
    CompatEntry {
        aibox_version: "0.34.3",
        processkit_version: "v0.28.8",
        note: "Patch release: restores readable Yazi marked items across every theme, repairs PowerKit separator colors, preserves active-pane emphasis, uses supported Codex lifecycle hooks, and reliably restores the outer terminal title.",
    },
    CompatEntry {
        aibox_version: "0.34.4",
        processkit_version: "v0.28.8",
        note: "Patch release: updates the bundled Yazi pane-toggle plugin to the current indexed ratio API and removes runtime deprecation warnings.",
    },
    CompatEntry {
        aibox_version: "0.34.5",
        processkit_version: "v0.28.8",
        note: "Patch release: restores PowerKit window-separator color continuity across every theme, makes isolated visual regressions mandatory for releases, and refreshes deferred tool pins.",
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
