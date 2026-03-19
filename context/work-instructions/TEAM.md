# Team Collaboration

## Agent Strategy

Three agent scopes for parallel work:

### Agent 1: Image Builder
**Scope:** All Dockerfiles in `images/`, this project's `.devcontainer/Dockerfile`.

### Agent 2: CLI Developer
**Scope:** Everything in `cli/`, `templates/`, `schemas/`.

### Agent 3: Documentation + Integration
**Scope:** `docs/`, `mkdocs.yml`, README.md, integration testing.

### Parallelization
- Image Builder + CLI Developer can run in parallel (CLI depends on knowing image names but not actual Dockerfiles)
- CLI Developer + Documentation can run in parallel
- All three can parallel on independent features

## Release Process

Full release workflow documented in `context/project-notes/release-process.md`.
Quick summary: all builds and deploys are local (no GitHub Actions).

1. Version bump (Cargo.toml + docs)
2. `./scripts/maintain.sh release X.Y.Z` (build + tag + release notes)
3. Cross-compile x86_64, push tag, create GitHub release
4. User runs macOS builds + pushes images on host
