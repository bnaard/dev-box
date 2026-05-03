# binding-management MCP server

Manage Binding entities — scoped, temporal, many-to-many relationships
between any two entities. Layer 2 — depends on `event-log` and
`actor-profile`.

This server documents its own tool contract only. In gateway deployments,
the gateway must expose only the processkit servers present in the
installed/merged MCP configuration; this file is not an aggregate tool
manifest.

## Tools

| Tool                                                                                                       | Purpose                                                  |
|------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| `create_binding(type, subject, target, scope?, valid_from?, valid_until?, conditions?, description?)`     | Create a new Binding under `context/bindings/`           |
| `create_time_window(subject, target, recurrence_rule_artifact, valid_from?, valid_until?, scope?, description?)` | Create a v2 schedule-demotion Binding using an Artifact-backed recurrence rule |
| `create_budget_application(cost_policy_artifact, target, enforcement_point, cap_usd?, scope?, valid_from?, valid_until?, description?)` | Bind a cost-policy Artifact to a target with budget metadata |
| `end_binding(id, end_date?)`                                                                              | Set `valid_until` (default: today)                       |
| `query_bindings(type?, subject?, target?, scope?, active_only?, limit?)`                                  | Query Bindings with filters; default returns active only |
| `resolve_bindings_for(entity_id, type?, at_time?)`                                                        | Find all Bindings whose subject or target is `entity_id` |

## Active vs ended

A binding is **active** at a time `t` if `valid_from ≤ t ≤ valid_until`
(treating absent endpoints as open). The default for `query_bindings` is
`active_only=True`. Pass `False` to see historical bindings.

## When to use a Binding vs a cross-reference

| Situation                                              | Use         |
|--------------------------------------------------------|-------------|
| "A blocks B"                                           | cross-ref   |
| "Alice is a developer" (globally)                     | cross-ref (in Actor) |
| "Alice is tech lead on project X for 2026"             | **Binding** |
| "Security gate applies to releases on main"            | **Binding** |

In v2, Process and Schedule are not first-class binding endpoints.
Bind the concrete governed entity instead, and use `create_time_window`
for schedule-like recurrence.

See `binding-management` SKILL.md for the rule and examples.

## Running

```bash
uv run context/skills/processkit/binding-management/mcp/server.py
```
