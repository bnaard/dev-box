---
name: aggregate-mcp
description: |
  Expose the processkit MCP tool surface through a single stdio server.
  Use for harnesses that eagerly start every configured MCP server, such
  as Codex, while keeping per-skill MCP servers available.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v2
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
      opt_in_config:
        - mcp/mcp-config.aggregate.json
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

The current runtime is an eager-import aggregate: importing
`aggregate-mcp/mcp/server.py` imports every processkit per-skill MCP
server and registers the discovered tools immediately. This is useful
for clients that spend memory and latency on many stdio processes, but
it is not yet a lazy daemon or gateway process that loads a backing
server only when one of its tools is called.

## Tool Names

Tools keep their original names when the name is unique across processkit.
If two source servers expose the same tool name, the aggregate server keeps
the first one and registers later duplicates as
`<skill_slug>__<tool_name>`. This preserves familiar names such as
`create_workitem` while keeping duplicate helpers like `reload_schemas`
available.

`list_aggregate_tools` is the registry surface for gateway adapters. It
reports each registered tool's aggregate name, source skill, source MCP
server path, source tool name, serialized annotations, collision status,
collision source list, and whether the aggregate name was deduplicated.
The response also includes runtime metadata:

- `runtime.import_mode: "eager"`
- `runtime.lazy_daemon: false`
- `source_server_count`

Collision status values are:

- `unique` - no other source server exports that tool name.
- `primary` - the first source server kept the unprefixed tool name.
- `namespaced_duplicate` - a later source server was registered with a
  source-skill prefix.

## Aggregate Mode Config

The opt-in fragment at `mcp/mcp-config.aggregate.json` registers only
`processkit-aggregate-mcp`:

```json
{
  "mcpServers": {
    "processkit-aggregate-mcp": {
      "command": "uv",
      "args": [
        "run",
        "context/skills/processkit/aggregate-mcp/mcp/server.py"
      ],
      "env": {
        "PROCESSKIT_MCP_MODE": "aggregate"
      }
    }
  }
}
```

Harness adapters such as aibox should merge this fragment instead of
the per-skill `mcp-config.json` fragments when aggregate mode is
enabled. It is intentionally not named `mcp-config.json`, so existing
granular installers do not register aggregate mode beside all granular
servers and increase startup work.

## Gotchas

- Do not remove the per-skill MCP servers. The aggregate server is an
  adapter for eager-start clients, not the canonical implementation.
- Harness adapters must opt into this server deliberately. Projects that
  rely on fine-grained MCP server permissions may still prefer per-skill
  registration.
- Duplicate helper tools are namespaced only when needed. Callers should
  prefer the original unprefixed names for unique tools.
- This is not yet a daemon gateway. The next runtime step is a long-lived
  gateway process that exposes the same registry shape but defers backing
  server import and heavier initialization until a tool is invoked.

## Full reference

The aggregate server lives at `mcp/server.py`. It intentionally does
not ship a default `mcp-config.json`, because registering it beside the
granular servers would increase startup work instead of reducing it.
Harness adapters can use `mcp/mcp-config.aggregate.json` as the opt-in
replacement fragment when they support aggregate mode:

```json
{
  "mcpServers": {
    "processkit-aggregate-mcp": {
      "command": "uv",
      "args": [
        "run",
        "context/skills/processkit/aggregate-mcp/mcp/server.py"
      ],
      "env": {
        "PROCESSKIT_MCP_MODE": "aggregate"
      }
    }
  }
}
```

The `list_aggregate_tools` tool reports the source skill, source server
path, source tool, annotations, and collision status for every exported
tool.
