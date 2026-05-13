# Runtime Prune MCP Server

Provider-neutral MCP wrapper for inspecting and invoking cleanup of
aibox-managed runtime state.

| Tool | Safety | Description |
|---|---|---|
| `analyze_disk_usage(project_root, scopes)` | Read-only | Inventory selected cleanup scopes and return size/risk data. |
| `plan_prune(project_root, scopes)` | Read-only | Produce a dry-run plan with expected reclaimed space and confirmation token. |
| `apply_prune(project_root, scopes, confirmation)` | Destructive | Run `aibox prune` for explicitly approved scopes and return evidence. |

`apply_prune()` requires the exact `required_confirmation` token returned
by `plan_prune()`. If `aibox prune` is not available on `PATH`, the tool
returns an error and does not delete anything itself.
