---
name: agent-management
description: Multi-agent workflow coordination including task delegation, context sharing, handoff protocols, and quality gates. Use when orchestrating multiple AI agents, designing agent pipelines, or managing agent-to-agent communication.
---

# Agent Management

## When to Use

When the user asks to:
- Coordinate multiple AI agents on a shared task
- Decompose a large task into agent-assignable subtasks
- Design an agent pipeline (research, implement, review)
- Set up handoff protocols between agents
- Define quality gates for multi-agent workflows
- Recover from agent failures or stalled tasks

## Instructions

### 1. Task Decomposition

Break work into agent-sized units. Each subtask must have:

1. **Clear scope** -- one well-defined outcome, not "do everything"
2. **Input specification** -- exactly what files, context, or prior outputs the agent needs
3. **Output specification** -- what artifact the agent produces (file, diff, structured answer)
4. **Success criteria** -- how to verify the subtask is done correctly
5. **Estimated complexity** -- small (single file), medium (module), large (cross-cutting)

Rules of thumb:
- If a subtask requires more than 3 files of context, split it further
- If a subtask has no verifiable output, redefine it
- Prefer depth-first (finish one subtask fully) over breadth-first (start many in parallel)

### 2. Agent Role Definitions

Assign roles based on the task, not on agent identity:

| Role | Responsibility | Input | Output |
|---|---|---|---|
| **Planner** | Decomposes task, defines subtask order and dependencies | User request, codebase overview | Task plan with ordered subtasks |
| **Researcher** | Gathers information, reads code, finds patterns | Specific questions, file paths | Structured findings with citations |
| **Coder** | Implements changes, writes code | Task spec, relevant files, findings | Code changes (diff or files) |
| **Reviewer** | Validates correctness, style, completeness | Code changes, task spec, success criteria | Approval or rejection with reasons |

Guidelines:
- A single agent can fill multiple roles sequentially (plan, then code, then self-review)
- Never skip the reviewer role -- even self-review catches errors
- The planner role is mandatory for tasks touching more than 2 files

### 3. Context Sharing Strategies

Agents lose context between sessions. Use explicit context transfer:

**File-based (preferred for Claude Code):**
- Write findings to a scratch file (e.g., `_agent_notes.md`) that the next agent reads
- Use structured formats: headings, bullet points, file paths with line numbers
- Delete scratch files when the workflow completes

**Structured handoff messages:**
- Summarize what was done, what remains, and what blockers exist
- Include exact file paths and line numbers, not vague descriptions
- List assumptions made and decisions taken

**Shared memory (project files):**
- Use `DECISIONS.md` or `HANDOVER.md` for persistent cross-session context
- Keep entries timestamped and concise
- Reference specific commits or file versions

### 4. Handoff Protocols

Every handoff between agent roles must include:

1. **Status** -- what is complete and what is not
2. **Artifacts** -- list of files created or modified, with paths
3. **Verification** -- what was checked and how (tests run, manual review, linting)
4. **Blockers** -- anything that prevents the next step
5. **Next action** -- explicit instruction for the receiving agent

Format for handoff:
```
## Handoff: [Role] -> [Role]
**Status:** [complete | partial | blocked]
**Changes:** [list of file paths]
**Verified:** [tests passed | linted | reviewed]
**Blockers:** [none | description]
**Next:** [specific instruction]
```

### 5. Quality Gates

Insert verification checkpoints between stages:

- **After planning:** Does the plan cover all requirements? Are subtasks independent?
- **After research:** Are findings specific (file paths, line numbers) not vague?
- **After coding:** Do tests pass? Does `cargo clippy` / linter pass? Does the change match the spec?
- **After review:** Are all must-fix items addressed? Is the reviewer satisfied?

A quality gate fails the workflow forward -- do not proceed until the gate passes. If a gate fails twice on the same issue, escalate to the user.

### 6. Error Recovery and Fallbacks

When an agent step fails:

1. **Capture the failure** -- save error output, not just "it failed"
2. **Diagnose** -- is it a context issue (missing info), a capability issue (wrong approach), or an environment issue (tool error)?
3. **Retry with adjustment** -- provide missing context, try alternative approach, or fix environment
4. **Escalate** -- if two retries fail, stop and report to the user with full context

Fallback strategies:
- If a coder agent is stuck, switch to researcher role first to gather more context
- If review finds fundamental issues, return to planner role to re-scope
- If context is too large, split the task into smaller independent pieces

### 7. Progress Tracking

For multi-step workflows, maintain a checklist:

```
## Agent Workflow: [Task Name]
- [x] Plan: decomposed into N subtasks
- [x] Research: gathered context for subtask 1
- [ ] Code: implement subtask 1
- [ ] Review: verify subtask 1
- [ ] Code: implement subtask 2
...
```

Update the checklist after each step. This serves as both progress tracking and handoff context.

## Examples

**User:** "Add authentication to this API -- use multiple agents to plan, implement, and review."
**Agent:** Creates a task plan: (1) Researcher reads existing route handlers and middleware patterns, writes findings to `_auth_notes.md`. (2) Coder implements auth middleware, JWT validation, and protected route decorators based on findings. (3) Reviewer checks the implementation against OWASP guidelines, verifies tests pass, and confirms no hardcoded secrets. Each handoff includes the structured handoff block with status, artifacts, and next action.

**User:** "Set up a code review pipeline for incoming changes."
**Agent:** Defines a three-stage pipeline: (1) Researcher reads the diff and gathers context about affected modules. (2) Reviewer evaluates correctness, security, performance, and style using the code-review skill checklist. (3) Planner summarizes findings into must-fix, should-fix, and nit categories with specific line references. Quality gate: all must-fix items require a re-review cycle before approval. See `references/coordination-patterns.md` for the pipeline pattern.

**User:** "Research the best caching strategy, then implement it."
**Agent:** Runs a research-then-implement workflow: (1) Researcher examines current data access patterns, identifies hot paths, checks existing caching (if any), and writes structured findings with measured or estimated latencies. (2) Planner reviews findings and selects a caching approach, documenting the decision. (3) Coder implements the cache layer following the plan. (4) Reviewer verifies cache invalidation logic, checks for stale data risks, and confirms tests cover cache hit/miss paths. Handoff between research and planning includes specific file paths and quantified findings.
