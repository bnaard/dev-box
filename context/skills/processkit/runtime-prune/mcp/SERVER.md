# Runtime Prune MCP Server

Provider-neutral MCP wrapper for inspecting runtime-manager-owned state
and applying only processkit-owned cleanup scopes.

| Tool | Safety | Description |
|---|---|---|
| `analyze_disk_usage(project_root, scopes)` | Read-only | Inventory selected cleanup scopes and return size/risk data. |
| `plan_prune(project_root, scopes)` | Read-only | Produce a dry-run plan with expected reclaimed space and confirmation token. |
| `apply_prune(project_root, scopes, confirmation)` | Destructive | Apply processkit-owned cleanup scopes and return external host-action evidence for unsupported scopes. |

`apply_prune()` requires the exact `required_confirmation` token returned
by `plan_prune()`. It never invokes host orchestrator commands from
inside the container.
