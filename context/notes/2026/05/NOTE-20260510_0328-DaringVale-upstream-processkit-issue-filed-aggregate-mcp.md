---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260510_0328-DaringVale-upstream-processkit-issue-filed-aggregate-mcp
  created: '2026-05-10T03:28:15+00:00'
spec:
  title: "Upstream processkit issue filed: aggregate-mcp lazy imports \u2014 https://github.com/projectious-work/processkit/issues/31"
  body: See Markdown body below.
  type: reference
  state: permanent
---
## Filed

Upstream processkit issue filed on 2026-05-10:

- **URL**: https://github.com/projectious-work/processkit/issues/31
- **Title**: `aggregate-mcp: support lazy per-skill module imports to reduce cold-start time`
- **Repo**: `projectious-work/processkit`

## Cross-reference

This closes the action item in the original tracking note:
`NOTE-20260510_0326-CuriousTrout-upstream-processkit-issue-tracker-aggregate-mcp`

The original note requested: "When capacity allows, file the upstream processkit issue. Once filed, update this Note with the issue URL and re-evaluate the dependent aibox WorkItem."

## Next steps

- Re-evaluate aibox companion WorkItem `BACK-20260510_0325-DaringAsh-aggregate-mcp-gateway-defer-per-skill` once processkit upstream responds or makes progress on issue #31.
- The aibox-side `[mcp.aggregate]` config knob (lazy mode flag) depends on the upstream runtime capability landing first.

## Context summary

aibox v0.25.7 shipped `McpGatewayMode::Aggregate` (commit 2837ca5) — a single stdio process replacing N per-skill MCP processes, eliminating Codex cold-start latency. The processkit-side `aggregate-mcp` server still imports all skill modules eagerly at startup. Issue #31 proposes deferring per-skill imports until first tool call referencing that skill (`lazy_catalog` mode).
