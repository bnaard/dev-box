---
name: aggregate-mcp
description: |
  Expose the processkit MCP tool surface through a single stdio server.
  Use for harnesses that eagerly start every configured MCP server, such
  as Codex, while keeping per-skill MCP servers available.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-aggregate-mcp
    version: "1.0.0"
    created: 2026-04-29T00:00:00Z
    category: processkit
    layer: 1
    uses:
      - skill: index-management
        purpose: Imported as one of the aggregate MCP tool providers.
    provides:
      mcp_tools: [list_aggregate_tools]
---

# Aggregate MCP

## Intro

This skill ships a single MCP server that imports the processkit per-skill
MCP servers and re-exports their tools through one stdio process. It is a
startup-latency optimization for harnesses that eagerly initialize every
configured MCP server.

Per-skill servers remain canonical and should continue to be shipped. The
aggregate server is an alternate entry point for clients that prefer one
process over many.

## Overview

Use this server when a harness pays a large startup cost for many stdio
MCP processes. The aggregate server imports the same tool functions that
the granular processkit servers expose, so the processkit write/read
semantics stay in one code path.

## Tool Names

Tools keep their original names when the name is unique across processkit.
If two source servers expose the same tool name, the aggregate server keeps
the first one and registers later duplicates as
`<skill_slug>__<tool_name>`. This preserves familiar names such as
`create_workitem` while keeping duplicate helpers like `reload_schemas`
available.

## Gotchas

- Do not remove the per-skill MCP servers. The aggregate server is an
  adapter for eager-start clients, not the canonical implementation.
- Harness adapters must opt into this server deliberately. Projects that
  rely on fine-grained MCP server permissions may still prefer per-skill
  registration.
- Duplicate helper tools are namespaced only when needed. Callers should
  prefer the original unprefixed names for unique tools.

## Full reference

The aggregate server lives at `mcp/server.py`. It intentionally does
not ship a default `mcp-config.json` yet, because registering it beside
the granular servers would increase startup work instead of reducing it.
Harness adapters should register this server instead of the per-skill
servers when they support aggregate mode:

```json
{
  "mcpServers": {
    "processkit-aggregate-mcp": {
      "command": "uv",
      "args": [
        "run",
        "context/skills/processkit/aggregate-mcp/mcp/server.py"
      ],
      "env": {}
    }
  }
}
```

The `list_aggregate_tools` tool reports the source skill and source
tool for every exported tool.
