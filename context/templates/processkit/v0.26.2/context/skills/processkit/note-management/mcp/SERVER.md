# note-management MCP server

Capture Notes and operate the v2 Hook inbox convention.

This server documents its own tool contract only. In gateway deployments,
the gateway must expose only the processkit servers present in the
installed/merged MCP configuration; this file is not an aggregate tool
manifest.

## Tools

| Tool | Purpose |
|---|---|
| `prepare_hook_inbox_dirs(base_dir?)` | Create `tasks/inbox`, `tasks/claimed`, `tasks/done`, and `tasks/failed` adapter hand-off directories |
| `create_note(title, body, type?, tags?, source?)` | Capture a Note entity |
| `capture_inbox_item(title, body, injection_mode?, channel?, source?, target_workitem?, tags?)` | Capture a Hook-inbox fleeting Note |
| `claim_inbox_item(id, actor?)` | Mark a captured inbox Note as claimed |
| `complete_inbox_item(id, actor, result?)` | Mark a claimed inbox Note as completed |
| `fail_inbox_item(id, actor, error)` | Mark a claimed inbox Note as failed |
| `reload_schemas()` | Clear this server's in-process schema + state-machine caches |

## Running

```bash
uv run context/skills/processkit/note-management/mcp/server.py
```
