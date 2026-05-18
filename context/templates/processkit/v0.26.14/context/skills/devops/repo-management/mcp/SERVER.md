# Repo Management MCP Server

Provider-neutral MCP wrapper for repository stewardship across local git
state, issues, and change requests.

| Tool | Safety | Description |
|---|---|---|
| `detect_repo_provider(project_root, remote)` | Read-only | Detect provider, host, owner/repo, and local CLI availability. |
| `inspect_repo_state(project_root, include_remote)` | Read-only | Inspect branch, upstream, dirty state, ahead/behind, remotes, and provider capabilities. |
| `list_repo_issues(state, limit)` | Read-only | List supported provider issues, currently GitHub via `gh`. |
| `list_repo_change_requests(state, limit)` | Read-only | List supported provider PRs/MRs/change requests, currently GitHub via `gh`. |
| `plan_repo_reconcile(scope, dry_run)` | Read-only | Build the reconciliation plan and blockers. |
| `resolve_repo_issue(id, resolution, comment, close, confirmation)` | Mutating | Comment on and optionally close a supported issue. |
| `merge_change_request(id, method, auto_queue, confirmation)` | Mutating | Merge or queue a supported ready change request. |
| `commit_local_changes(paths, message, checks)` | Mutating | Run optional checks, stage paths, and commit local changes. |
| `push_current_branch(remote, branch)` | Mutating | Push the current branch without force. |
| `run_repo_reconcile(dry_run, max_items, commit_message, push)` | Mutating | Apply guarded batch actions and return per-action evidence. |

Mutating issue and merge operations require exact confirmation tokens:
`close-issue:<id>` and `merge-change-request:<id>`. The server never
force-pushes and never bypasses provider checks.
