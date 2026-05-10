---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0325-DaringAsh-aggregate-mcp-gateway-defer-per-skill
  created: '2026-05-10T03:25:41+00:00'
  labels:
    version: v0.25.7-followup
    area: mcp-aggregate
    depends: processkit-upstream
  updated: '2026-05-10T11:24:04+00:00'
spec:
  title: 'aggregate-mcp gateway: defer per-skill module imports until first tool call
    (lazy_catalog)'
  state: done
  type: task
  priority: medium
  description: |
    ## Background

    v0.25.7 shipped `McpGatewayMode::Aggregate` (commit 2837ca5, merge 9a35160) — a single stdio process that imports all per-skill MCP modules in-process, replacing N stdio handshakes with 1. RoyalHawk's report flagged a residual cost: the aggregate server currently imports every skill module at startup (eager).

    A truly lazy variant would defer per-skill imports until the first tool call referencing that skill. The processkit `aggregate-mcp` skill's SKILL.md already documents this as the "next runtime step" but has not implemented it.

    ## Goal

    Reduce the aggregate server's cold-start time by adding a `lazy_catalog` mode that only imports a skill's module when one of its tools is called.

    ## Scope

    - aibox-side: a `[mcp.aggregate]` config option that the aibox-rendered `mcp-config.aggregate.json` propagates to the aggregate server.
    - processkit-side: a separate upstream issue tracks the runtime change. See sibling tracking note.

    ## Open questions

    - Should lazy_catalog be a flag or a different gateway mode (e.g. `AggregateLazy`)?
    - How to surface tool discovery (ListTools) without paying the full eager cost? Cached snapshot vs. per-skill on-demand probe?

    ## Acceptance

    - aibox emits the new config knob in mcp-config.aggregate.json when set.
    - A timing test (or manual measurement) shows cold-start delta before/after.

    ## Refs

    - Commit 2837ca5 (RoyalHawk's aggregate mode)
    - Sibling tracking note for upstream processkit issue: see Note created same day.
  started_at: '2026-05-10T11:23:48+00:00'
  completed_at: '2026-05-10T11:24:04+00:00'
---

## Transition note (2026-05-10T11:24:04+00:00)

Implemented and merged in commit 7cae4d9 + merge 6bb317b. New McpGatewayMode::LazyAggregate variant in cli/src/config.rs; lazy_aggregate_mcp_spec() helper in cli/src/mcp_registration.rs emits PROCESSKIT_MCP_MODE=lazy_catalog. Auto chain unchanged (opt-in only). Companion to upstream processkit#31. 904 tests pass.
