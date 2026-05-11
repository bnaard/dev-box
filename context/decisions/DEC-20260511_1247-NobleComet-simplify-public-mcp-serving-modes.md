---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1247-NobleComet-simplify-public-mcp-serving-modes
  created: '2026-05-11T12:47:49+00:00'
spec:
  title: Simplify public MCP serving modes
  state: accepted
  decision: 'aibox will expose processkit MCP serving as four public modes: auto,
    daemon, stdio, and separate. Legacy/internal mode names daemon-proxy and granular
    remain accepted as aliases for compatibility. Aggregate and lazy-aggregate are
    demoted from the main public configuration surface; lazy behavior is controlled
    by lazy_catalog where the selected topology supports it.'
  context: The user observed that the product model should be daemon gateway, stdio
    gateway, and fallback separate MCP servers, with auto selecting between them.
    The existing enum exposed accumulated implementation variants and fallback modes
    that made the configuration harder to understand.
  rationale: Simpler public vocabulary matches how users reason about MCP serving
    while preserving existing project configs. Keeping aliases avoids breaking old
    aibox.toml files, and demoting aggregate variants avoids hiding implementation
    experiments in the main UX.
  consequences: Config serialization, docs, status labels, logs, doctor text, and
    tests need to use the simplified public names. Internal fallback code can still
    use aggregate support where useful, but user-facing surfaces should not present
    it as a primary mode.
  decided_at: '2026-05-11T12:47:49+00:00'
---
