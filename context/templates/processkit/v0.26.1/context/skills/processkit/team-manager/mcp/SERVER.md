# team-manager MCP server

Manages TeamMember entities — persistent participants (humans, named
AI personas, services) with identity, personality, and tiered memory.
Replaces the deprecated actor-profile server.

## Tools

### Lifecycle

| Tool | Purpose |
|---|---|
| `create_team_member(name, type, slug, default_role?, default_seniority?, personality?, email?, handle?, joined_at?)` | Create a new TeamMember entity. Auto-reserves name if `type=ai-agent`. |
| `get_team_member(id)` | Fetch a single TeamMember by ID. |
| `list_team_members(type?, active_only?, role?, limit?)` | List TeamMembers with optional filters. |
| `update_team_member(id, **fields)` | Update mutable fields on an existing TeamMember. |
| `deactivate_team_member(id, left_at?)` | Mark inactive (sets active=false + left_at). |
| `reactivate_team_member(id)` | Undo deactivation (active=true, clears left_at). |

### Session identity

| Tool | Purpose |
|---|---|
| `get_active_interlocutor(scope?)` | Return the configured TeamMember speaker for a harness session, if any. |
| `get_interlocutor_runtime_binding(scope?, observed_model?, observed_effort?, task_hints?)` | Return the active interlocutor plus the resolved model binding, harness capability mode, and informational mismatch report. |
| `set_active_interlocutor(id, scope?)` | Configure the TeamMember speaker for a scope by writing `context/team/session-identity.json`. |

### Name pool

| Tool | Purpose |
|---|---|
| `reserve_name(name, team_member_slug)` | Mark a pool name reserved for a slug. |
| `release_name(name)` | Drop a reservation. |
| `list_available_names(kind?)` | List unreserved names, optionally filtered by gender bucket (`feminine`/`masculine`/`neutral`). |
| `suggest_name(kind?)` | Pick one unreserved name at random. |

### Memory tree

| Tool | Purpose |
|---|---|
| `init_memory_tree(slug)` | Create tier subdirectories, `.gitkeep` files, and scaffolded `persona.md` + `card.json` + `team-member.md` under `context/team-members/<slug>/`. |

### Export / import

| Tool | Purpose |
|---|---|
| `export_team_member(slug, output_path?)` | Build a tar.gz bundle with persona + card + entity + knowledge/ + skills/ + lessons/. Excludes journal/, relations/, private/. Redacts confidential/pii memory files. |
| `export_claude_subagent(slug, output_dir?, overwrite?, model_policy?)` | Generate `.claude/agents/<slug>.md` for one TeamMember using Claude Code's subagent adapter format. `model_policy` defaults to `inherit`; `resolved` emits a Claude model only when routing resolves to Anthropic. |
| `export_claude_subagents(output_dir?, active_only?, include_humans?, overwrite?, model_policy?)` | Generate Claude Code subagent adapters for active non-human TeamMembers by default. |
| `import_team_member(tarball_path)` | Extract, validate entity + Agent Card, require signature field present (crypto verification deferred), copy into `context/team-members/<slug>/`. |

### Consistency

| Tool | Purpose |
|---|---|
| `check_consistency(slug)` | 10 checks on a single team-member; returns structured findings. |
| `check_all_consistency()` | Aggregated findings across every team-member. |

## Storage

TeamMember entities live at
`<project-root>/context/team-members/<slug>/team-member.md`. Their ID
is `TEAMMEMBER-<slug>`.

Session identity lives at `<project-root>/context/team/session-identity.json`.
It is deliberately separate from TeamMember lifecycle state.

Runtime binding is capability-negotiated. processkit reports the
provider-neutral resolved candidate and harness capability mode, but it
does not hot-swap an already running primary harness session. When the
caller supplies `observed_model` or `observed_effort`, mismatches are
returned as informational warnings for session-start display.

## Running

```bash
uv run context/skills/processkit/team-manager/mcp/server.py
```

## Notes

Deprecates `actor-profile` per DEC-20260422_0233-SpryTulip. AI-agent
names are drawn from `data/name-pool.yaml`; reservations are managed
by `reserve_name` / `release_name` with atomic file rewrites.

For ephemeral role+seniority dispatches (no persistence), use
`task-router` — TeamMembers are only for persistent personas.
