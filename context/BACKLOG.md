# Backlog

Central task registry. Each item has a unique ID for cross-referencing.
Source of truth — GitHub issues are for external visibility.
Archive of completed/merged items: [BACKLOG-ARCHIVE.md](BACKLOG-ARCHIVE.md)

## Next ID: BACK-043

## Format

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|

Status values: `todo`, `in-progress`, `done`, `blocked`, `archived`
Priority values: `must`, `should`, `could`, `wont`

---

## Active Items

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-002 | Security review | todo | must | #23 — input validation, container security, supply chain. Sequenced after architecture overhaul (BACK-022) and CLI overhaul (BACK-028) |
| BACK-004 | Skill eval framework | todo | should | Test/benchmark skills per Anthropic skill-creator pattern |
| BACK-006 | Docs review (existing-project, base-image pages) | todo | could | Deprioritized — do after Docusaurus migration (BACK-021) and architecture changes settle |
| BACK-007 | Plugin / extension system | todo | should | #20 — hooks, template overrides, community features |
| BACK-008 | Zellij plugin integration (zjstatus) | todo | should | #21 — custom aibox status plugin; needs further discussion with owner |
| BACK-010 | Evaluate multi-service support approach | todo | could | Analysis only: should we support multi-service or leave to user? Factor in SSH/remote dev plans |
| BACK-011 | Remote development | todo | should | Environments on remote hosts, local thin client |
| BACK-012 | Session handover format | todo | should | Standard process template for AI session continuity; may already be done — needs review and gap analysis |
| BACK-014 | Binary checksum verification | todo | must | Verify downloaded binaries in base Dockerfile. Sequenced after architecture overhaul |
| BACK-015 | Image signing (cosign) | todo | must | sigstore/cosign for published images. Sequenced after architecture overhaul |
| BACK-016 | Skill security vetting | todo | must | Hash verification, allowed-tools audit. Sequenced after architecture overhaul |
| BACK-017 | LaTeX addon package groups | todo | could | Add music (lilypond, musixtex), chemistry (chemfig, mhchem), linguistics tool groups to the latex add-on definition in addon_registry.rs |
| BACK-018 | AIUC-1 compliance alignment | todo | could | Awareness for aibox |
| BACK-019 | Skill marketplace integration | todo | could | ClawHub, Skills.sh |
| BACK-020 | `aibox doctor` skill consistency | todo | could | Check installed vs declared skills |
| BACK-024 | External skill installation | todo | could | Allow installing skills from sources outside curated list. Deprioritized. |
| BACK-025 | Skills gap analysis — internet research | todo | should | Research most-used agent skills/categories externally; research common SE/infra/docs/research tasks; compare with our 83 skills; identify gaps |
| BACK-026 | Existing skills quality review | todo | should | Audit all skills for missing examples, code snippets, and tools per SKILL.md format. Evaluate where tools would improve reliability (algorithmic vs probabilistic) |
| BACK-027 | Three-level document structure audit | todo | should | Investigate whether all documents (project + derived project templates) follow the SKILL.md three-level rule (intro → overview → details) for context-efficient consumption by agents |
| BACK-028 | Complete CLI/UX overhaul | todo | must | Merge of BACK-005 + BACK-023. kubectl as reference model. See expanded scope below |
| BACK-029 | CLI output formats (table/JSON/YAML) | todo | could | Add structured output formats to list commands. Deprioritized, investigate later |
| BACK-030 | Bring-your-own-model AI provider support | todo | could | Allow custom/self-hosted model endpoints (e.g., vast.ai) in [ai] config. Deferred. |
| BACK-031 | Revise process bundles from competitive research | todo | should | After BACK-022 implementation, revisit context/ideas/competitive-landscape-2026-03.md and process-frameworks-research.md to refine process packages and extract ideas (e.g., event log pattern) |
| BACK-032 | Clean up context/project-notes directory | todo | should | After BACK-022 implementation, consolidate scattered files in project-notes/; valuable content but too unstructured |
| BACK-033 | Context ideas research review | todo | should | After BACK-022 implementation, review all files in context/ideas/ for actionable insights not yet captured in backlog |
| BACK-034 | New skill: software modularization | todo | should | Skill on keeping software in small, independent packages optimized for AI agent context limits. Covers module boundaries, package decomposition, API surface design for agent comprehension |
| BACK-035 | New skill: microservice creation & orchestration | todo | should | Skill on creating new services/microservices and orchestrating them. Service boundaries, inter-service communication, deployment patterns |
| BACK-039 | Develop visual identity | todo | must | Research and create brand identity: logo (SVG, multiple sizes), tagline/claim, color palette (for web + docs), page design vibe, font selection (headings + body), favicon. Produce 4-5 alternative concepts, then select one. Informs Docusaurus theme, README, social preview image |
| BACK-040 | Analyse base image Dockerfile for multistage build optimization | todo | should | Current Dockerfile has 2 stages (builder for binary downloads, runtime). Analyse opportunities to further reduce final image size: separate apt-get layers (core utils vs preview deps vs audio), split binary downloads into individual stages for better cache invalidation, evaluate whether vim colorscheme downloads can be a separate stage, measure current image size and estimate savings. Consider impact on build time vs image size tradeoff. **Key target:** Node/Bun are only needed for Docusaurus docs builds — investigate using a dedicated builder stage to run the docs build and copy only the static output, avoiding Node/Bun runtime in the final image entirely. **Also investigate:** non-root user in dev-containers (user "aibox") — security best practice, affects file ownership on mounts, PATH, home directory location |
| BACK-042 | Internal project site for context documents | todo | could | Investigate and design a project-internal site that renders all markdown files in `./context/` as a browsable, nicely formatted site. Could be extended to a GitHub-independent wiki. **Intent:** collaboration for future multiple human workers, better readability of context files. Options: lightweight static site generator (e.g., Docusaurus second instance, mdBook, wiki.js), or extend existing docs-site with a context section. Should be simple to start — just rendered markdown with navigation |

### BACK-028 Expanded Scope

Complete CLI/UX overhaul with kubectl as the reference model:

1. **Command simplification**: Merge build/attach/generate into `aibox sync` as the primary reconciliation command. Reduce command surface area.
2. **`aibox skill` subcommands**: Imperative interface to browse, add, remove, list curated skills. Updates aibox.toml declaratively.
3. **`aibox addon` subcommands**: Imperative interface to add, remove, list add-ons (Python, Node, Go, LaTeX, etc.) with version selection. Updates aibox.toml.
4. **kubectl-style UX patterns**: Resource-oriented commands (`aibox get`, `aibox describe`?), consistent verb patterns, discoverable help. Investigate what makes sense for aibox — not a blind copy, but learn from kubectl's ergonomics.
5. **Relationship to BACK-022**: The CLI overhaul implements the user-facing interface for the declarative config model designed in BACK-022. BACK-022 (architecture) should be resolved first, then BACK-028 (CLI) follows.
