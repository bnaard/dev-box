---
sidebar_position: 1
title: "Skills Library"
---

# Skills Library

aibox ships **84 curated skills** across 14 categories. Every skill follows the open [SKILL.md standard](https://agentskills.io/specification) and is deployed into `.claude/skills/` based on your process packages and addons.

## What Are Skills?

A skill is a directory containing a `SKILL.md` file (and optional `references/` files) that teaches an AI agent how to perform a specific task. Skills use **progressive disclosure**:

1. **Metadata** (~100 tokens) -- `name` and `description` loaded at startup for all skills
2. **Instructions** (<5000 tokens) -- full `SKILL.md` body loaded when the skill activates
3. **References** (on demand) -- detailed reference files loaded only when needed

```
.claude/skills/
├── kubernetes-basics/
│   ├── SKILL.md              # Main instructions
│   └── references/
│       ├── cluster-architecture.md
│       ├── resource-cheatsheet.md
│       └── troubleshooting.md
├── code-review/
│   └── SKILL.md              # Simple skill, no references
└── ...
```

## Categories at a Glance

| Category | Skills | Description |
|----------|--------|-------------|
| [Process](process.md) | 9 | Backlog, decisions, standups, releases, incidents, retrospectives, agent coordination |
| [Development](development.md) | 11 | Code review, testing, debugging, refactoring, error handling, documentation |
| [Language](language.md) | 7 | Python, Rust, TypeScript, Go, Java, SQL style, LaTeX |
| [Infrastructure](infrastructure.md) | 10 | Docker, Kubernetes, DNS/networking, Terraform, Linux, shell scripting, CI/CD |
| [Architecture](architecture.md) | 4 | Software architecture, DDD, event-driven, system design |
| [Design & Visual](design.md) | 7 | Frontend, Tailwind, Excalidraw, infographics, logos, PixiJS, mobile UX |
| [Data & Analytics](data.md) | 5 | Data science, pipelines, visualization, feature engineering, data quality |
| [AI & ML](ai-ml.md) | 6 | AI fundamentals, RAG, prompt engineering, LLM evaluation, embeddings, ML pipelines |
| [API & Integration](api.md) | 4 | REST API design, GraphQL, gRPC/Protobuf, webhooks |
| [Security](security.md) | 5 | Auth patterns, secure coding, threat modeling, dependency audit, secrets |
| [Observability](observability.md) | 4 | Logging, metrics, distributed tracing, alerting |
| [Database](database.md) | 4 | SQL patterns, data modeling, NoSQL, migrations |
| [Performance](performance.md) | 4 | Profiling, caching, concurrency, load testing |
| [Framework & SEO](framework.md) | 5 | FastAPI, Reflex, pandas/polars, Flutter, SEO |

## How Skills Are Deployed

Skills are deployed based on three sources, merged in order:

### 1. Process Packages

Your `[process].packages` config determines the base skill set. Each package bundles related skills:

```toml
[process]
packages = ["managed"]  # Expands to: core, tracking, standups, handover
```

The **13 packages** and their skills:

| Package | Skills |
|---------|--------|
| `core` | agent-management, owner-profile |
| `tracking` | backlog-context, decisions-adr, event-log, context-archiving |
| `standups` | standup-context |
| `handover` | session-handover, inter-agent-handover |
| `product` | estimation-planning, retrospective |
| `code` | code-review, testing-strategy, debugging, refactoring, tdd-workflow, error-handling, git-workflow, integration-testing |
| `research` | data-science, data-visualization, feature-engineering |
| `documentation` | documentation, latex-authoring |
| `design` | excalidraw, infographics, logo-design, frontend-design, mobile-app-design |
| `architecture` | software-architecture, system-design, domain-driven-design, event-driven-architecture |
| `security` | secure-coding, threat-modeling, dependency-audit, auth-patterns, secret-management, dependency-management |
| `data` | data-pipeline, data-quality, pandas-polars, embedding-vectordb |
| `operations` | ci-cd-setup, dockerfile-review, container-orchestration, logging-strategy, metrics-monitoring, incident-response, alerting-oncall, performance-profiling |

**4 convenience presets** expand to multiple packages:

| Preset | Expands to |
|--------|-----------|
| `managed` | core, tracking, standups, handover |
| `software` | core, tracking, standups, handover, code, architecture |
| `research-project` | core, tracking, standups, handover, research, documentation |
| `full-product` | core, tracking, standups, handover, code, architecture, design, product, security, operations |

### 2. Addon Skills (automatic)

When you add an addon, its recommended skills are automatically included. No manual `[skills].include` needed:

| Addon | Auto-deployed skills |
|-------|---------------------|
| `python` | python-best-practices, fastapi-patterns, pandas-polars |
| `rust` | rust-conventions, concurrency-patterns |
| `go` | go-conventions, concurrency-patterns |
| `node` | typescript-patterns, tailwind |
| `latex` | latex-authoring, documentation |
| `typst` | documentation |
| `kubernetes` | kubernetes-basics, container-orchestration |
| `infrastructure` | terraform-basics |
| `docs-*` (all 6) | documentation |

For example, adding the `python` addon to a `managed` project automatically deploys `python-best-practices`, `fastapi-patterns`, and `pandas-polars` alongside the managed package skills.

### 3. Manual Include/Exclude

Fine-tune with `[skills]` in `aibox.toml`:

```toml
[skills]
include = ["flutter-development", "seo-optimization"]  # Add extra skills
exclude = ["standup-context"]                            # Remove unwanted ones
```

Core skills (`agent-management`, `owner-profile`) cannot be excluded.

## Managing Skills via CLI

```bash
# List all skills and their deploy status
aibox skill list

# Add a skill (updates aibox.toml skills.include)
aibox skill add flutter-development

# Remove a skill (adds to skills.exclude)
aibox skill remove standup-context

# View skill details
aibox skill info rust-conventions
```

### Updating Skills

When you upgrade aibox, new skills are added automatically on the next `aibox sync`. Existing skills are never overwritten -- only missing ones are created.

## Custom Skills

Create your own skills alongside the bundled ones:

```markdown
# .claude/skills/my-custom-skill/SKILL.md
---
name: my-custom-skill
description: What this does and when to use it. Be specific to help trigger detection.
allowed-tools: Bash(npm:*) Read Write
---

# My Custom Skill

## When to Use
Describe trigger conditions.

## Instructions
Step-by-step agent instructions.

## Examples
Scenario-based examples.
```

## SKILL.md Format

Every skill follows the [Agent Skills specification](https://agentskills.io/specification):

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Lowercase kebab-case, max 64 chars |
| `description` | Yes | What it does AND when to use it, max 1024 chars |
| `allowed-tools` | No | Pre-approved tools (e.g., `Bash(kubectl:*) Read Write`) |
| `license` | No | License identifier |
| `compatibility` | No | Environment requirements |
| `metadata` | No | Arbitrary key-value pairs |

### Progressive Disclosure Best Practices

- Keep `SKILL.md` under 500 lines (ideally 50-150)
- Move detailed reference material to `references/*.md`
- Agents load references on demand, keeping context lean
- Include 2-4 scenario-based examples in every skill

## Security

:::warning Only install skills from trusted sources

Skills contain instructions that AI agents execute. A malicious skill could instruct an agent to modify files or exfiltrate data. Only install skills from sources you trust.

:::

- **Review before installing** -- read the SKILL.md file before adding third-party skills
- **Version control** -- commit `.claude/skills/` to git so changes are tracked
- **Prefer bundled** -- the 84 skills shipped with aibox are curated and maintained
- **Audit external skills** -- treat them like any dependency
