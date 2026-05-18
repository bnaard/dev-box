# artifact-management MCP server

Artifact registration, retrieval, and update. Layer 2 — depends on
`event-log` and `index-management`.

This server documents its own tool contract only. In gateway deployments,
the gateway must expose only the processkit servers present in the
installed/merged MCP configuration; this file is not an aggregate tool
manifest.

## Tools

| Tool | Purpose |
|---|---|
| `create_artifact(name, kind, location?, format?, version?, owner?, produced_by?, tags?, description?)` | Register a new Artifact under `context/artifacts/` |
| `get_artifact(id)` | Fetch one Artifact with full spec |
| `query_artifacts(kind?, owner?, tags?, limit?)` | List Artifacts matching filters |
| `update_artifact(id, name?, kind?, location?, format?, version?, owner?, produced_by?, tags?, touch_updated_at?)` | Update metadata fields on an existing Artifact |
| `reload_schemas()` | Clear in-process schema + state-machine caches so a disk edit is picked up without a server restart (returns `{ok, cleared: {schemas, state_machines}}`). Scope: this server only. PEP 723 dep edits still require a harness restart. See DEC-QuickPine. |

## Valid kinds

`document`, `design`, `dataset`, `build`, `slides`, `video`, `spec`,
`diagram`, `url`, `other`

## Conventions

- Artifacts have no state machine — create and update are the only
  write operations.
- `produced_at` is auto-stamped on creation.
- `location` is required for pointer artifacts (content lives outside
  the repository). Omit for self-hosted artifacts whose content is
  the Markdown body.
- `update_artifact` only modifies fields you supply; omitted fields
  are preserved.
- Pass `touch_updated_at=false` for metadata-only updates that should
  not restamp `metadata.updated` and invalidate an agent's prior
  body-read freshness checks.

## Running

```bash
uv run context/skills/processkit/artifact-management/mcp/server.py
```
