---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260411_1554-CleverAsh-gitignore-mcp-json-and
  created: '2026-04-11T15:54:07+00:00'
  updated: '2026-04-18T12:00:00Z'
spec:
  title: Gitignore .mcp.json and all generated MCP configs; add [mcp] sections to aibox.toml and .aibox-local.toml
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-04-11T15:55:54+00:00'
---

## Transition note (2026-04-11T16:50:09+00:00)

Implemented: ExtraMcpServer + McpSection in config.rs; mcp field in AiboxConfig + AiboxLocalConfig; local_mcp_servers populated at merge time; regenerate_mcp_configs merges processkit + team + personal specs (all in managed set); .mcp.json and all harness configs added to gitignore template, ensure_aibox_entries, check_gitignore_entries, and project .gitignore.

## Transition note (2026-04-18T12:00:00Z) — review -> done

Verified during the aibox#53 implementation session (2026-04-18). The
gitignore entries for `.mcp.json`, `.cursor/mcp.json`, and
`.continue/mcpServers/` are present at /workspace/.gitignore lines 17,
18, 21 (with the explanatory comment block at lines 15-16). The
`[mcp.servers]` schema sections in `aibox.toml` / `.aibox-local.toml`
and the three-source merge in `mcp_registration.rs::regenerate_mcp_configs`
are also confirmed present. The aibox#53 fix in v0.18.6 makes this
machinery actually exercise itself for the first time on a real install
(previously the broken walker prevented any merge from happening), so
this WorkItem is now end-to-end validated.

Marked done by claude-opus-4-7.
