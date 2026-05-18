# Runtime Prune MCP Server

Provider-neutral MCP wrapper for inspecting runtime-manager-owned state,
repo-local generated artifacts, and remote quota surfaces. It applies
only processkit-owned local cleanup scopes; host runtime and provider
cleanup is returned as external host-action evidence.

| Tool | Safety | Description |
|---|---|---|
| `analyze_disk_usage(project_root, scopes)` | Read-only | Inventory selected cleanup scopes and return size/risk data. |
| `plan_prune(project_root, scopes)` | Read-only | Produce a dry-run plan with expected reclaimed space, confirmation token, and executable external runbooks. |
| `apply_prune(project_root, scopes, confirmation)` | Destructive | Apply processkit-owned cleanup scopes and return external host-action evidence for unsupported scopes. |

`apply_prune()` requires the exact `required_confirmation` token returned
by `plan_prune()`. It never invokes host orchestrator, forge/provider,
package registry, Docker, or Podman deletion commands from inside the
container.

Supported scopes are `runtime-home`, `build-cache`, `repo-artifacts`,
`tool-caches`, `agent-worktrees`, `containers`, `e2e-companion`,
`action-artifacts`, `release-assets`, and `package-registry`.

For external provider scopes, `plan_prune()` includes `required_env`,
`inventory_command`, `dry_run_command`, `apply_command`, and
`space_estimate`. The commands are intended for a derived-project agent
or human to run after setting the required variables and reviewing the
dry-run output.
