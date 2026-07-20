---
argument-hint: ""
allowed-tools: []
---

Use the status-briefing skill to generate a session-start orientation
and catch-up summary.

Before writing the briefing, query `migration-management` for pending
and in-progress migrations. If any exist, resolve them before normal
priorities unless the user explicitly says `info-only`, `dry-run`,
`report-only`, `check-only`, or `no fixes`.

Use the migration-management MCP tools; never move migration files by
hand. Apply or continue unambiguous migrations. Reject malformed no-op
or empty-baseline migrations only when the defect and rejection reason
are clear. Ask before applying or rejecting a migration that changes
policy, deletes data, has unclear intent, conflicts with local work, or
needs a product/domain decision. Re-query migrations after remediation
and include the final active count in the briefing.

Also call the direct `run_pk_doctor` MCP tool and include its ERROR/WARN/INFO
totals, `action_totals`, and top `action_required: true` findings. Treat
severity and actionability as separate queues: `0 WARN` is not clean when
actionable INFO findings remain unresolved. If actionable findings exist, ask
for or report a disposition: fix, migrate, archive, link tracking, defer with
reason, or accept as a policy exception. If the tool is unavailable while
`pk-doctor` is enabled, report that as an MCP configuration problem instead of
silently falling back to a local script.

If the workspace is in a git repository with a GitHub remote, check open
GitHub issues and pull requests via `gh` when available/authenticated. Include
review-needed PRs, stale or high-priority issues, and external blockers; note
briefly if GitHub could not be checked.
