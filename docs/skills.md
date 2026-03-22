# Skills

Skills provide executable instructions for AI agents. They complement [process templates](context/overview.md#process-templates) by defining **HOW** a task should be performed, while processes define **WHAT** tasks exist.

## What Are Skills?

A skill is a `SKILL.md` file that tells an AI agent how to perform a specific task. Skills follow the open [SKILL.md standard](https://agentskills.io/specification) -- a lightweight format with YAML frontmatter (name, description) and Markdown instructions.

The separation of WHAT (processes) from HOW (skills) is a core dev-box design decision (DEC-011). This enables:

- **Swappable implementations** -- replace a context-file skill with a GitHub-integrated one without changing your process declarations
- **Testable skills** -- each skill has a clear scope and can be validated independently
- **Composable workflows** -- mix and match skills from different sources

## How Skills Are Installed

Skills live in `.claude/skills/<name>/SKILL.md`. Each skill gets its own directory:

```
.claude/
└── skills/
    ├── backlog-context/
    │   └── SKILL.md
    ├── decisions-adr/
    │   └── SKILL.md
    └── standup-context/
        └── SKILL.md
```

Claude Code automatically discovers skills in this directory structure. When a user request matches a skill's "when to use" criteria, the AI agent follows the skill's instructions.

## Bundled Skills

dev-box includes three example skills that work with the context system:

### backlog-context

Manages the project backlog as a `BACKLOG.md` file in the context directory. Handles creating, prioritizing, and tracking work items in Markdown format.

- **When to use:** When the user asks to add, update, or review backlog items
- **Artifact:** `context/BACKLOG.md`
- **Format:** Checkbox items grouped by priority (Next Up, Planned, Ideas)

### decisions-adr

Manages architectural decision records in `context/DECISIONS.md`. Records decisions with rationale, alternatives considered, and implications.

- **When to use:** When the user makes a significant technical or process decision
- **Artifact:** `context/DECISIONS.md`
- **Format:** Numbered entries (DEC-NNN) in inverse chronological order

### standup-context

Manages session standup notes in `context/STANDUPS.md`. Records what was done, what is planned, and any blockers.

- **When to use:** At the start of a new session, or when the user asks to record progress
- **Artifact:** `context/STANDUPS.md`
- **Format:** Date-headed entries with Done/Next/Blockers sections

## Relationship to Processes

Processes and skills work together:

1. **Process templates** (`context/processes/`) declare requirements: "there shall be code review"
2. **Skills** (`.claude/skills/`) provide implementation: instructions for the AI agent on how to perform code review
3. **Context artifacts** (`context/`) store the results: BACKLOG.md, DECISIONS.md, STANDUPS.md, etc.

Skills come in flavors. For example, backlog management could be implemented by:

- `backlog-context` -- manages a Markdown file in `context/BACKLOG.md`
- `backlog-github` -- manages GitHub Issues (not yet available)

You choose the flavor that fits your workflow. The process declaration stays the same regardless of which skill implements it.

## Creating Custom Skills

To create a custom skill:

1. Create a directory: `.claude/skills/<your-skill-name>/`
2. Add a `SKILL.md` file with YAML frontmatter and instructions:

```markdown
---
name: my-custom-skill
description: Brief description of what this skill does.
---
# My Custom Skill

## When to use

Describe when the AI agent should activate this skill.

## Instructions

Step-by-step instructions for the AI agent.
```

## Open Standard

The SKILL.md format follows the open specification at [agentskills.io/specification](https://agentskills.io/specification). This ensures skills are portable across tools that support the standard.

## Security

!!! warning "Only install skills from trusted sources"
    Skills contain instructions that AI agents execute. A malicious skill could instruct an agent to modify files, exfiltrate data, or perform other harmful actions. Only install skills from sources you trust -- review the SKILL.md content before adding it to your project.

Best practices:

- **Review before installing** -- read the SKILL.md file before adding it to your project
- **Version control skills** -- commit `.claude/skills/` to git so changes are tracked
- **Prefer bundled skills** -- the skills shipped with dev-box are vetted and maintained
- **Audit third-party skills** -- treat external skills like any other dependency: review the source
