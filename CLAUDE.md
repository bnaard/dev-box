# CLAUDE.md — dev-box

**dev-box** is a CLI tool analogous to **uv for AI work environments**. It unifies
reproducible containerized development environments with built-in AI context structure.

**Key facts:** Rust CLI (`cli/`), 8 published container images (`images/`),
4 work process flavors (`templates/`), MkDocs docs (`docs/`).
No GitHub Actions — all builds and deploys are local.

## Critical: .devcontainer/ vs images/

We are in a dev-container building dev-containers. **Never confuse these two:**

- **`.devcontainer/`** — THIS project's own dev environment (Rust + Python/uv + MkDocs)
- **`images/`** — Published images for OTHER projects (pushed to GHCR)

## Building and Testing

```bash
cd cli && cargo build                     # Build CLI
cd cli && cargo test                      # Run all tests (unit + integration)
cd cli && cargo clippy -- -D warnings     # Lint (must pass with zero warnings)
```

## Project Structure

See `context/work-instructions/DEVELOPMENT.md` for detailed structure and config spec.

## Context Files

The `context/` directory contains structured project management files:

- `context/OWNER.md` — Profile of the project owner
- `context/DECISIONS.md` — Decision log (inverse chronological, DEC-NNN)
- `context/work-instructions/GENERAL.md` — General rules, known issues, gotchas
- `context/work-instructions/DEVELOPMENT.md` — Build, test, project structure, config spec
- `context/work-instructions/TEAM.md` — Agent strategy, release process summary
- `context/project-notes/release-process.md` — Full release checklist with Phase 0 dependency check

## GitHub Organization

- **Repo:** `projectious-work/dev-box`
- **GHCR:** `ghcr.io/projectious-work/dev-box`
- **Docs:** `https://projectious-work.github.io/dev-box/`
