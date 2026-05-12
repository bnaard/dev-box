---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260510_0326-CuriousTrout-upstream-processkit-issue-tracker-aggregate-mcp
  created: '2026-05-10T03:26:20+00:00'
spec:
  title: "Upstream processkit issue tracker \u2014 aggregate-mcp lazy module imports"
  body: See Markdown body below.
  type: reference
  state: permanent
---
## Why this Note exists

RoyalHawk's v0.25.7 implementation of `McpGatewayMode::Aggregate` (commit 2837ca5) eliminated N-process Codex startup latency on the aibox side. A complementary improvement on the processkit side — making `aggregate-mcp` truly lazy at module-import time — would further reduce first-tool latency for users opting into Aggregate mode.

This is upstream processkit work, not aibox work. Tracking here so it doesn't get lost.

## What to file at processkit

Open an issue at the processkit upstream repo with these notes:

- Currently `context/skills/processkit/aggregate-mcp/` does eager import of every per-skill MCP module at server startup. The SKILL.md Gotchas section already documents this as the next runtime step.
- Proposal: defer per-skill module imports to first tool-call referencing that skill. Maintain a snapshot of advertised tools (cached at startup or on first ListTools) so discovery stays fast.
- Co-changes:
  - aibox-side `[mcp.aggregate]` config knob to opt into lazy mode (tracked in sibling aibox WorkItem).
  - Some thought needed about ListTools behavior under lazy: cached snapshot vs. per-skill on-demand probe.

## Action

When capacity allows, file the upstream processkit issue. Once filed, update this Note with the issue URL and re-evaluate the dependent aibox WorkItem (aggregate-mcp lazy_catalog).

## Refs

- Aibox commit 2837ca5 (`feat(mcp): add aggregate gateway mode`)
- Aibox WorkItem: aggregate-mcp gateway lazy_catalog (sibling, just created)
- Processkit skill: `context/skills/processkit/aggregate-mcp/SKILL.md`
