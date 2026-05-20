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
| 0.27.2 | v0.27.0 | fixes the `docs-hugo` addon checksum verification so Hugo archives downloaded to `/tmp/hugo.tar.gz` are checked against the matching release checksum entry instead of the upstream asset filename |
| 0.27.1 | v0.27.0 | refreshes addon and user-facing toolchain pins, including documentation generators, language package managers, infrastructure tooling, Kubernetes tools, Helm, and OpenCode; adds release-state coverage for addon pins plus LaTeX and apt-managed inputs; keeps Python interpreter selection tied to the Debian base image package set; fixes OpenCode release asset naming and checksum verification; updates docs-site dependencies and clears npm audit findings |
| 0.27.0 | v0.27.0 | integrates processkit v0.27.0 and the v0.26.17 supply-chain audit surface; switches the next-minor GHCR image scheme to foundation/runtime tags, stops publishing public source-hash marker tags, preserves legacy `base-debian-v0.26.x` compatibility, adds GHCR source-tag cleanup tooling, adds release LICENSE guardrails, and fixes pasted host newlines by removing global tmux `C-j` navigation |
| 0.26.8 | v0.26.16 | integrates processkit v0.26.16; refreshes the processkit template mirror, provenance, MCP manifest, TeamMember privacy defaults, and team consistency checks; makes latest image resolution skip GHCR tags whose multi-arch manifest children have been pruned |
| 0.26.7 | v0.26.15 | integrates processkit v0.26.15; adds provider-neutral AI execution policy axes and per-harness overrides, maps execution policy to Codex settings, preserves the processkit MCP manifest in derived installs, and refreshes documentation recordings |
| 0.26.6 | v0.26.14 | integrates processkit v0.26.14; updates processkit metadata to the new release and preserves tmux/Vim/cheatsheet and clipboard behavior consistency |
| 0.26.5 | v0.26.13 | re-architects theme selection around theme families plus mode/variant, adds the 61-theme gallery and per-theme recordings, ships PowerKit/cheatsheet/runtime theme fixes, and preserves legacy concrete theme intent during standardization |
| 0.26.4 | v0.26.10 | integrates processkit v0.26.10; makes explicit tmux model-provider status elements render even without global provider polling; preserves Claude Code OAuth/state across container rebuilds via Claude XDG cache/config/state mounts; enables terminal extended-key passthrough for Alt-Enter; adds PowerKit GitHub issue/PR counts; and fixes `aibox-status` AI agent counting so vendor helper processes do not inflate provider instance totals |
| 0.26.3 | v0.26.9 | fixes GHCR latest-version resolution by requesting larger tag-list pages and following Docker Registry v2 pagination links so freshly-published images are visible to `aibox apply` |
| 0.26.2 | v0.26.9 | fixes duplicate `[customization.tmux.status]` table parsing, routes tmux helper scripts through the managed socket, removes structurally broken capture-pane visual tests, and keeps visual layout/theme work tracked for asciinema coverage |
| 0.26.1 | v0.26.9 | verifies GHCR release-host image pushes, fixes Yazi rich-preview API/cache behavior, suppresses recurring legitimate processkit template mirror repair warnings, emits migration affected files, and integrates processkit v0.26.9 |
| 0.26.0 | v0.26.7 | refreshes themes, live tmux layout/theme choosers, Yazi rich preview, Vim Alt-key handling, model-provider agent counts, statusline structure, and processkit v0.26.7 integration |
| 0.25.14 | v0.26.5 | integrates processkit v0.26.5; restores generated and image fallback Alt-word movement in Vim/readline; adds managed `.inputrc` runtime projection; carries tmux clipboard/terminal feature improvements into generated and image fallback configs; keeps Rust cache mounts from shadowing image-provided cargo/rustc shims; and keeps the stricter processkit schema/file-layout migration path clean under pk-doctor |
| 0.25.13 | v0.26.2 | fixes stale runtime-home propagation by making aibox-managed `.aibox-home` files authoritative on apply and clean runtime recreation; broadens generated runtime mounts for Vim and Cargo cache directories; updates generated compose/docs coverage; and adds regression coverage for stale tmux/Yazi managed file refresh |
| 0.25.12 | v0.26.2 | fixes fresh-project tmux/PowerKit runtime projection by using broad writable runtime-home mounts, refreshing managed theme/Yazi/Codex files on apply, scoping preauth/MCP writes to enabled harnesses, recognizing nested processkit skill catalogs in doctor, and reducing host-side doctor probes for container-owned dependencies |
| 0.25.11 | v0.26.2 | fixes source-checkout addon discovery, PowerKit status cache writability, generated runtime writability diagnostics, disabled-harness migration schema metadata, and tmux/Yazi status glyph defaults |
| 0.25.10 | v0.26.2 | integrates processkit v0.26.2; adds configurable tmux status labels/layouts and model-provider health segments; reduces PowerKit refresh churn; stabilizes runtime MCP diagnostics; verifies release checksum sidecars; and improves image layer/cache reuse across apply/up and host release publishing |
| 0.25.9 | v0.26.1 | integrates processkit v0.26.1; retires yazi-omp runtime support; migrates tmux status configuration to list-based slot ordering; preserves user-selected themes during standardization; suppresses Kubernetes/cloud PowerKit auth/probe flashes; and improves Yazi directory git status behavior |
| 0.25.8 | v0.26.0 | improves the tmux log viewer, filters aibox log counts to the current container session, emits low-volume diagnostics sidecar lifecycle samples, moves PowerKit metrics into the owner-specified two-row layout, and updates generated tmux layouts around ordered harness semantics |
| 0.25.7 | v0.25.8 | introduces generated-container cleanup controls, prunes SSH companion nested runtime state around Tier 2 E2E tests, adds `aibox prune`, and documents managed runtime cleanup policy |
| 0.25.6 | v0.25.8 | expands tmux/PowerKit theme coverage with light/dark partner handling, generated theme comments, Docusaurus theme documentation, and runtime border/status color improvements |
| 0.25.5 | v0.25.8 | refreshes managed tmux runtime files when recreating sessions so `aibox.toml` status/layout settings take effect, preserves the delayed Yazi pane startup path, and suppresses stale default-socket tmux kill-session noise on host attach |
| 0.25.4 | v0.25.8 | repairs tmux release-host smoke probing after the managed tmux socket migration and fixes generated tmux status-right rendering by preserving the aibox runtime status segment |
| 0.25.3 | v0.25.8 | fixes Yazi `e` editor handoff for tmux by targeting the existing Vim pane/window, documents tmux status modes and element toggles, makes skill-finder discover deselected skills from the template catalog, and tightens SSH companion guidance |
| 0.25.2 | v0.25.8 | adds tmux-native `prefix ?` keybinding popup, upgrades the two-line PowerKit status with pane context/mode detail, labels status metrics in `aibox-status`, fixes startup layout targeting by selecting named tmux windows instead of `$session:1`, extends release/runtime caching behavior, and documents provider endpoint base URL hints (`ANTHROPIC_BASE_URL`, `OPENAI_BASE_URL`, `GEMINI_BASE_URL`, `MISTRAL_BASE_URL`) |
| 0.25.0 | v0.25.8 | replaces the prior multiplexer runtime with tmux-native layouts and status, keeps the diagnostics sidecar and visual testing gates, documents TPM as a user convenience layer only, preinstalls and pins aibox-managed tmux plugins, and ships `tmux-resurrect`/`tmux-continuum` installed but disabled by default until persistence policy is decided |
| 0.24.1 | v0.25.8 | fixes generated compose so the main service starts with the image default root entrypoint user again, allowing `entrypoint.sh` to remap/drop to `aibox` instead of failing with `failed switching to "aibox": operation not permitted` during release-host runtime smoke |
| 0.24.0 | v0.25.8 | adds the bounded diagnostics sidecar, replaces the shell fan-out `aibox-status` helper with Rust snapshot readers, wires sidecar-backed Zellij status rows, adds `aibox emergency <harness>`, keeps legacy `native`/`hidden` status aliases while emitting `sidecar`/`disabled`, and reduces host release smoke to a minimal default tier with opt-in addon/full tiers |
| 0.23.21 | v0.25.8 | repairs generated Yazi git/status initialization for Yazi 26; preserves native Zellij plugin permission caches across runtime starts; adds doctor and E2E guardrails for native Zellij permission-cache projection drift; installs the Yazi `ya` companion entrypoint in runtime images; slims visual E2E release gates with per-case progress logging and an opt-in exhaustive matrix |
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
