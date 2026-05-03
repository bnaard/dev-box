---
name: processkit-gateway
description: |
  Provider-neutral processkit MCP gateway. Use when a harness should see
  one processkit MCP surface while processkit keeps per-skill servers
  canonical and available.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v2
    id: SKILL-processkit-gateway
    version: "0.1.0"
    created: 2026-05-02T00:00:00Z
    category: processkit
    layer: 1
    uses:
      - skill: aggregate-mcp
        purpose: Compatibility mode and legacy one-process entrypoint.
    provides:
      mcp_tools: [list_gateway_tools, gateway_health]
      mcp_config: mcp/mcp-config.json
---

# Processkit Gateway

## Intro

This skill exposes processkit's MCP tool surface through a provider-neutral
gateway entrypoint. It is additive: per-skill MCP servers remain canonical,
and `aggregate-mcp` remains available as the compatibility bridge for
existing harness configs.

## Overview

### Runtime modes

The default mode is an eager stdio gateway. It registers the same tool
functions as the per-skill processkit MCP servers and reports richer
metadata through `list_gateway_tools`.

The gateway also owns the long-lived daemon and proxy CLI shape:

- `serve --transport stdio`
- `serve --transport streamable-http`
- `stdio-proxy --url ...`
- `catalog --write`

`serve --transport streamable-http` starts a local streamable HTTP MCP
daemon. It binds to `127.0.0.1:8000/mcp` by default. `stdio-proxy` is a
lightweight stdio bridge for harnesses that cannot connect to HTTP MCP
servers directly. The proxy must not import source processkit MCP servers.

Set `PROCESSKIT_GATEWAY_IMPORT_MODE=lazy-catalog` or
`PROCESSKIT_GATEWAY_LAZY=true` to use the catalog-backed lazy registration
path. Generate the catalog with `catalog --write` before enabling lazy mode.

### aibox boundary

aibox may install, configure, start, and stop this gateway in managed
devcontainers. processkit must remain usable without aibox: users can run
the gateway server directly from the installed `context/skills` tree or
configure a harness to launch the stdio command.

## Gotchas

- Do not remove per-skill MCP servers; they are the compatibility and
  permission-granularity baseline.
- Do not expose tools globally without per-connection policy. Tool
  annotations are the first permission signal, not the whole policy model.
- Do not duplicate entity validation in the gateway. Delegate writes to the
  canonical management tool functions that already validate schemas, enforce
  state machines, and log side effects.
- Do not expose the streamable HTTP daemon on a non-local interface unless a
  deployment layer adds explicit authentication and network policy.

## Full reference

### CLI contract

- `serve --transport stdio`
- `serve --transport streamable-http`
- `stdio-proxy --url ...`
- `catalog --write`

### Environment

- `PROCESSKIT_GATEWAY_IMPORT_MODE=lazy-catalog`
- `PROCESSKIT_GATEWAY_LAZY=true`

### Provided MCP tools

- `list_gateway_tools`
- `gateway_health`
