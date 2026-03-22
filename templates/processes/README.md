# Processes

Process declarations define WHAT your project does — not HOW.

Each `.md` file in this directory describes a process: its steps, roles, and
definition of done. Processes are intentionally thin. The executable details
(formatting conventions, tool integrations, workflow specifics) live in
**skills** (see `.claude/skills/`).

## How it works

1. **Processes** declare requirements: "there shall be code review."
2. **Skills** provide implementation: a SKILL.md with instructions for the AI agent.
3. **Context artifacts** store the results: BACKLOG.md, DECISIONS.md, etc.

## Customizing

- Edit any process file to match your team's workflow.
- Add new process files for project-specific workflows.
- Remove processes you don't use.
- Install matching skills with `dev-box skill install <skill-name>` (coming soon).
