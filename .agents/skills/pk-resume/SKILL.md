---
name: pk-resume
description: Use the status-briefing skill to generate a session-start orientation
---

Use the status-briefing skill to generate a session-start orientation
and catch-up summary.

Before writing the briefing, query `migration-management` for pending
and in-progress migrations. If any exist, surface them before normal
priorities and propose the next action: review, apply, continue, or
reject via the migration-management MCP tools. Do not apply or reject a
migration unless the user explicitly asks.

Also call the direct `run_pk_doctor` MCP tool and include its ERROR/WARN/INFO
totals plus top actionable findings. If the tool is unavailable while
`pk-doctor` is enabled, report that as an MCP configuration problem instead of
silently falling back to a local script.

If the workspace is in a git repository with a GitHub remote, check open
GitHub issues and pull requests via `gh` when available/authenticated. Include
review-needed PRs, stale or high-priority issues, and external blockers; note
briefly if GitHub could not be checked.
