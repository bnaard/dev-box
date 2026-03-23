# CLAUDE.md — aibox

**aibox** is a CLI tool analogous to **uv for AI work environments**. It unifies
reproducible containerized development environments with built-in AI context structure.

**Key facts:** Rust CLI (`cli/`), 10 container image flavors (`images/`),
4 work process flavors (`templates/`), Docusaurus docs (`docs-site/`).
No GitHub Actions — all builds and deploys are local.

## Critical: .devcontainer/ vs images/

We are in a dev-container building dev-containers. **Never confuse these two:**

- **`.devcontainer/`** — THIS project's own dev environment (Rust + Python/uv + Docusaurus)
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

The `context/` directory follows the **product** process template:

- `context/BACKLOG.md` — Active task registry with BACK-NNN IDs (source of truth)
- `context/archive/` — Archived items (completed backlog, old decisions, completed projects, old session notes)
- `context/PROJECTS.md` — Project registry with PROJ-NNN IDs
- `context/PRD.md` — Product requirements document
- `context/DECISIONS.md` — Decision log (inverse chronological, DEC-NNN)
- `context/OWNER.md` — Profile of the project owner
- `context/work-instructions/GENERAL.md` — General rules, known issues, gotchas
- `context/work-instructions/DEVELOPMENT.md` — Build, test, project structure, config spec
- `context/work-instructions/TEAM.md` — Agent strategy, release process summary
- `context/project-notes/release-process.md` — Full release checklist with Phase 0 dependency check

## GitHub Organization

- **Repo:** `projectious-work/aibox`
- **GHCR:** `ghcr.io/projectious-work/aibox`
- **Docs:** `https://projectious-work.github.io/aibox/`
