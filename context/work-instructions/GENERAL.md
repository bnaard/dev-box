# General Work Instructions

Machine-managed by aibox. These rules apply to all AI agents working in this repo.

## Critical Architectural Distinction

**We are in a dev-container building dev-containers.**

This repo has TWO separate sets of container definitions:

- **`.devcontainer/`** — THIS project's own dev environment (Rust toolchain, Python/uv/MkDocs). What VS Code and `scripts/dev.sh` use.
- **`images/`** — Published images that OTHER projects consume (pushed to GHCR). They do NOT include Rust toolchain or MkDocs.

Never confuse these two. Changes to `.devcontainer/` affect our development. Changes to `images/` affect downstream projects.

## Provider Independence

All project state must be stored in `./context/` — never in provider-specific locations (e.g., `.claude/memory/`, `.aider/`). This ensures:
- Any AI agent (Claude, Aider, Gemini, etc.) can pick up where another left off
- Session handovers, decisions, and project notes are committed to git
- No lock-in to a specific AI provider's memory system

**Session handovers** go to `context/archive/sessions/` (use the session-handover skill format).

## Communication

- Be concise and direct
- Always reference GitHub issue numbers in commits (fixes #N, refs #N)
- No trailing summaries after completing work

## Code Quality

- Zero clippy warnings with `-D warnings`
- All tests must pass before committing
- Run `cargo test` and `cargo clippy -- -D warnings` before commits

## Git Workflow

- Commit messages: conventional commits (feat:, fix:, chore:, docs:)
- Never force-push to main
- Include Cargo.lock in version bump commits

## Known Issues and Gotchas

- **Podman compose** output format varies by version; always use `inspect`
- **Stale image cache**: rebuild with `--no-cache` if container exits immediately
- **`.aibox-home/` must be in `.gitignore`** — contains SSH keys and personal config
- **Zellij pinned to 0.43.1**: change `ARG ZELLIJ_VERSION` to upgrade
- **`host.docker.internal`**: works on Docker Desktop and Podman pasta; bare Linux Docker may need `--add-host`
- **OrbStack virtiofs**: files mounted from macOS may lose execute permissions. Workaround: `chmod +x` inside container.
- **Claude Code OAuth in containers**: use `claude setup-token` or authenticate on host (credentials shared via `.claude` mount). See [anthropics/claude-code#14528](https://github.com/anthropics/claude-code/issues/14528). Do NOT use `network_mode: host`.
- **OrbStack network dropout**: After ~20 minutes of idle, OrbStack's VM NAT can drop connections — breaking Telegram plugins and other long-lived network features. Fix: set `keepalive = true` in `[container]` of `aibox.toml`. This adds a lightweight DNS lookup every 2 minutes via `postStartCommand`.
