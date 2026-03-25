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

The `context/` directory follows the **product** process template.
Rule: single chronological file → root, multiple files of same kind → subdirectory.

- `context/BACKLOG.md` — Active task registry with BACK-NNN IDs (source of truth)
- `context/DECISIONS.md` — Decision log (inverse chronological, DEC-NNN)
- `context/PROJECTS.md` — Project registry with PROJ-NNN IDs
- `context/PRD.md` — Product requirements document
- `context/OWNER.md` — Profile of the project owner
- `context/work-instructions/` — Reference docs (DEVELOPMENT, GENERAL, TEAM, RELEASE-PROCESS, ARCHITECTURE, etc.)
- `context/research/` — Research reports (competitive landscape, skill marketplace, visual identity)
- `context/brand/` — Brand package (design tokens, templates — may move to company repo)
- `context/archive/` — Archived items, mirrored structure (sessions/, plans/, notes/, plus BACKLOG/DECISIONS/PROJECTS)

## GitHub Organization

- **Repo:** `projectious-work/aibox`
- **GHCR:** `ghcr.io/projectious-work/aibox`
- **Docs:** `https://projectious-work.github.io/aibox/`
