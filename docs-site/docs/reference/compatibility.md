---
sidebar_position: 99
title: "Compatibility"
---

# Compatibility

## aibox ↔ processkit Version Matrix

Each aibox release is tested against a specific processkit version. The table
below shows the minimum compatible processkit version for each aibox release.

| aibox version | Min. processkit | Notes |
|--------------|-----------------|-------|
| 0.23.21 | v0.25.8 | repairs generated Yazi git/status initialization for Yazi 26; preserves native Zellij plugin permission caches across runtime starts; installs the Yazi `ya` companion entrypoint in runtime images; slims visual E2E release gates with per-case progress logging and an opt-in exhaustive matrix |
| 0.23.20 | v0.25.8 | makes the release runtime smoke harness host-safe by defaulting to shell Zellij status mode, capturing raw TUI output into logs instead of streaming escape sequences to the host terminal, and asserting on structured probe markers rather than terminal transcripts |
| 0.23.19 | v0.25.8 | hardens generated runtime startup by keeping Vim eager while disabling its startup cursor-position probe; removes suspended generated AI panes; pre-seeds native Zellij plugin permissions; fixes service-specific Codex bubblewrap seccomp fallback; updates Yazi git/preview config; adds generated-runtime and opt-in visual E2E release gates |
| 0.23.18 | v0.25.8 | updates generated Yazi config and theme filetype rules for Yazi 26's url/mime matcher schema; provides writable XDG state mounts for lazygit and similar TUIs; records follow-up runtime diagnostics and host-phase runtime smoke work |
| 0.23.17 | v0.25.8 | installs Claude Code from Anthropic's signed apt repository with a stable `/usr/local/bin/claude` path; makes the native aibox Zellij status/key-hint plugin the generated default; starts shell and lazygit tabs hot across layouts; refreshes Zellij, Yazi, uv, and Cargo dependencies; improves release-state reporting and harness version-pin support |
| 0.23.16 | v0.25.8 | moves Claude processkit command shims to Claude Code's current Skills layout, cleans legacy managed `.claude/commands` files, fixes native Zellij key-hint rendering, keeps Vim editor panes hot for Yazi edit handoff, and adds a pre-release dependency/harness state report |
| 0.23.15 | v0.25.8 | fixes a 0.23.14 `--standardize-config` regression where a blank `[ai.harness.<name>]` table could re-enable a commented-out harness; standard config rewrites also restore the standard processkit skill list instead of leaving every skill commented |
| 0.23.14 | v0.25.8 | canonical generated `aibox.toml` now uses `[ai.harness.<name>]` tables instead of the compact harness list; `aibox apply --standardize-config` performs an opt-in schema-clean canonical rewrite; stale/deprecated generated comments were removed; Yazi `e` again opens files in the dedicated Vim pane/tab |
| 0.23.13 | v0.25.8 | fixes 0.23.11-to-0.23.12 generated-config upgrades where a moved tool, such as `gh`, still sits under its old addon owner; `aibox apply` now migrates misplaced addon tool entries to their unique current catalog owner before strict validation and comment refresh |
| 0.23.12 | v0.25.8 | processkit v0.25.8 Xiaomi MiMo model-routing content and cleanup-hint provenance, native aibox Zellij key/status bar refinements, semantic AI/audio config sections, stable Claude CLI install path, addon tool validation, and stale processkit-managed skill detection |
| 0.23.11 | v0.25.7 | grouped `aibox.toml` schema around aibox, container, processkit, and ai sections; catalog-style AI harness/model-provider controls; generated path settings; product skill defaults; and managed Zellij status runtime repair |
| 0.23.10 | v0.25.7 | processkit v0.25.7 model-routing content, apply-time `aibox.toml` structure migration, self-documenting generated config comments, addon-backed image-slimming switches, and generated-runtime release finalization |
| 0.23.9 | v0.25.6 | restores shell-backed Zellij status rows as the default, hardens `aibox-status` against `/proc` races, fixes Yazi edit actions, applies addon dependency fallback handling to `aibox up`, and integrates processkit v0.25.6 provider-neutral pk command projections |
| 0.23.8 | v0.25.5 | native aibox Zellij status plugin now exports the literal WASM entrypoints Zellij loads, uses theme-default readable foreground text, and has no-container E2E coverage for the load/visibility regression |
| 0.23.7 | v0.25.5 | host-generated Codex processkit-gateway MCP paths now target the devcontainer workspace mount, preserving subagent-safe absolute paths without leaking host-only paths; doctor warns about stale host-side Codex MCP script paths |
| 0.23.6 | v0.25.5 | processkit v0.25.5 active interlocutor runtime binding, subagent MCP lifecycle guardrails, Codex MCP path fixes, addon fallback migrations, doctor schema/runtime-template diagnostics, lazygit-disabled cleanup, native Zellij status visibility, and stronger E2E coverage |
| 0.23.5 | v0.25.4 | generated Dockerfile lazygit disablement cleanup no longer aborts when lazygit is absent as an apt package, while still removing inherited lazygit binaries |
| 0.23.4 | v0.25.4 | processkit v0.25.4 gateway stdio-proxy daemon startup fixes, Codex pre_tool_use hook generation, Zellij status presentation control, and stale status-layout runtime sync repair |
| 0.23.3 | v0.25.3 | processkit v0.25.3 model-spec/model-profile migrations and Codex seccomp fallback for bubblewrap |
| 0.23.2 | v0.25.1 | processkit v0.25.1 model-recommender lifecycle metadata, task suitability classes, task-class-aware routing, and refreshed model roster |
| 0.23.1 | v0.25.0 | stale runtime cleanup for old Compose project names, lazygit disablement fixes, and procps runtime diagnostics |
| 0.23.0 | v0.25.0 | processkit v0.25.0 gateway integration, daemon-proxy mode, runtime pressure diagnostics, init reaping, optional git UI tools, and profile-aware environment metadata |
| 0.22.0 | v0.24.0 | processkit v0.24.0: context archiving, richer model routing metadata, semantic task-router scoring, archive-aware index metadata |
| 0.21.2 | v0.23.1 | processkit v0.23.1 release-audit cleanup and skill metadata fixes |
| 0.21.1 | v0.23.0 | processkit v0.23.0 model governance and release-audit integration |
| 0.21.0 | v0.22.0 | multi-harness slash-command scaffolding and content-diff safety fixes |
| 0.20.0 | v0.22.0 | processkit install integrity, preauth merge, and no-container scaffold mode |
| 0.19.0 | v0.21.0 | MCP permission configuration and processkit v0.21 content integration |
| 0.17.16 | v0.13.0 | **BREAKING**: rename `providers = ["codex"]` → `["openai"]`; fix zellij `--layout` flag; fix Rust x86_64 cross-compile target |
| 0.17.15 | v0.13.0 | MCP config model, zjstatus hints, Zellij Ctrl+q, processkit v0.13.0 |
| 0.17.5 | v0.8.0 | processkit v0.8.0 GrandLily src/ restructure |
| 0.17.4 | v0.6.0 | content migration documents (pending/in-progress/applied) |
| 0.17.3 | v0.6.0 | Claude Code slash-command adapters |
| 0.17.2 | v0.6.0 | core skill enforcement, processkit v0.6.0 compat |
| 0.17.0 | v0.5.0 | aibox.lock sectioned format |
| 0.16.1 | v0.4.0 | sync auto-install added |
| 0.16.0 | v0.4.0 | initial processkit integration |

## How compatibility is enforced

`aibox apply` compares the `[processkit].version` in your `aibox.toml` against
the minimum required version for the running aibox binary. If the pinned
processkit version is older than the minimum, a warning is emitted:

```
Warning: processkit v0.24.0 is below the minimum recommended version v0.25.0 for aibox v0.23.0 ...
```

This is a warning, not an error — older processkit versions can still install
successfully when their layout is still supported. The warning is a nudge to
upgrade, not a blocker.

## Upgrading processkit

To upgrade processkit in an existing project:

1. Edit `aibox.toml`:
   ```toml
   [processkit]
   version = "v0.25.5"
   ```

2. Run `aibox apply` on the host — the 3-way diff will show changed content
   and generate processkit content migration documents in `context/migrations/pending/`.

3. Review and apply the pending migrations.
