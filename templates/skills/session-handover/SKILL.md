---
name: session-handover
description: Generates a structured session handover note at the end of a work session. Use when the user says "handover", "wrap up", "end of session", "hand off", or asks to document what was done so another agent can pick up.
---

# Session Handover

## When to Use

Trigger this skill when the user says:
- "handover", "wrap up", "end of session", "end session"
- "write a handover note", "summarise what we did", "document progress"
- "hand off to the next agent"

Also trigger proactively when a session is ending and no handover note exists for today.

## Instructions

### 1. Gather context

Run these commands to understand the current state:

```
git log --oneline -20          # recent commits
git status                     # any uncommitted changes
git diff --stat HEAD~1         # what changed in the last commit
```

Read:
- `context/BACKLOG.md` — identify in-progress items
- Any files that were recently edited (from git log)

### 2. Fill in the template

Use the template at `templates/managed/session-handover.md` (or `templates/product/session-handover.md` for product process projects). Fill in each section:

- **Summary** — one bullet per meaningful unit of work completed
- **Open Threads** — anything started but not finished; each needs a next action
- **Decisions Made** — key choices with rationale (so next agent doesn't re-debate them)
- **Blockers** — concrete blockers with owner; write "None." if clear
- **Next Steps** — ordered list, most important first
- **Context for Next Agent** — branch, files changed, current state, gotchas

### 3. Save the file

Save to:
```
context/archive/sessions/session-handover-YYYY-MM-DD.md
```

Use today's date. If a file already exists for today, append `-2` to the filename.

### 4. Update BACKLOG.md

If any items changed status this session (todo → in-progress, in-progress → done), update the backlog table now.

## Example

```markdown
# Session Handover — 2026-03-26

## Summary

- Implemented BACK-012: session-handover template in managed and product templates
- Added session-handover skill to templates/skills/
- Updated context.rs to use the real template content for session_template_md

## Open Threads

- BACK-026 (skills quality review) — next action: audit context-archiving and standup-context skills first

## Decisions Made

- Decision: session-handover template is the same for managed and product (same structure, same fields). Rationale: handovers are session-level, not process-level — no meaningful difference between the two.

## Blockers

None.

## Next Steps

1. Run cargo clippy and cargo test to verify no regressions
2. Start BACK-026 skills quality review

## Context for Next Agent

**Branch / version:** main, v0.12.0
**Files changed this session:**
  templates/managed/session-handover.md (new)
  templates/product/session-handover.md (new)
  templates/skills/session-handover/SKILL.md (new)
  cli/src/context.rs (updated session_template_md content)
  context/BACKLOG.md (BACK-012 → done)
**Current state:** All tests passing, clippy clean.
**Gotchas:** The session_template_md key in context.rs seeds the template into new projects via aibox init. The content must be a valid fill-in-the-blanks document, not a filled example.
```
