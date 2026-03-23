# Product Requirements Document

## Vision

dev-box is a CLI tool for reproducible, containerized development environments
with built-in AI context structure. It unifies container management, opinionated
tooling, color theming, and structured work processes into a single binary —
giving solo developers and small teams a "batteries included" dev environment
that works the same everywhere.

## Target Users

- **Solo developers** using AI-assisted workflows who want reproducible,
  themed, opinionated environments without manual Docker/devcontainer setup
- **Small teams** needing consistent environments across members with
  structured context (backlogs, decisions, standups) for AI agent collaboration
- **Consultants and contractors** who spin up project environments frequently
  and need fast, repeatable scaffolding

## Core Requirements

- Single static binary CLI (Rust), no runtime dependencies on the host
- Container-based isolation via Docker/Podman devcontainers
- `dev-box.toml` configuration: container, context, AI, addons, appearance, audio
- Context scaffolding with 4 work process flavors (minimal, managed, product, research)
- Curated skill library (83+ skills) with SKILL.md + reference files
- Color theming across Zellij, Vim, Yazi, lazygit (6 themes)
- Three IDE layouts: dev, focus, cowork
- AI provider flexibility: Claude, Aider, Gemini — optional, stackable
- Addon bundles: infrastructure, kubernetes, cloud providers, documentation tools
- Named environment management (`dev-box env`)
- Config reconciliation (`dev-box sync`)
- Security scanning (`dev-box audit`)

## Non-Goals

- Full CI/CD platform — dev-box is for local development, not pipelines
- Kubernetes orchestrator — single-container focus, multi-service is future
- IDE replacement — dev-box provides the terminal environment, not the editor
- Enterprise multi-tenant — designed for individual/small team use
- Cloud hosting — environments run locally (remote dev is a future goal)

## Success Metrics

- Adoption by all projectious.work projects as the standard dev environment
- Community forks and contributions as signal of external value
- Time-to-productive for new projects: under 5 minutes from `dev-box init` to working
- Zero "works on my machine" issues across team members
