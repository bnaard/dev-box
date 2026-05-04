---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260504_1652-SnowyRobin-adopt-grouped-aibox-configuration-schema
  created: '2026-05-04T16:52:55+00:00'
spec:
  title: Adopt grouped aibox configuration schema
  state: accepted
  decision: Group aibox-owned settings under [aibox], container generation/build settings
    under [container], processkit context settings under [processkit], and AI harness/agent/MCP
    settings under [ai]. Deprecate processkit package selection in favor of the full
    product skill set with explicit skill enable/disable catalog semantics.
  context: 'The rendered aibox.toml catalog was useful but exposed old schema boundaries:
    metadata, image, context, agents, and mcp were separate top-level sections. The
    owner accepted a cleaner grouped structure and requested implementation plus a
    patch release.'
  rationale: The grouped schema matches user mental models, makes generated config
    easier to scan, removes the increasingly stale package abstraction, and harmonizes
    catalogs around comment/uncomment selection.
  consequences: Legacy keys need backward-compatible aliases and migration. The renderer
    should emit the new structure, while parser support should continue for old metadata.name,
    image.version/base, context.packages/schema_version, agents, and mcp during transition.
  decided_at: '2026-05-04T16:52:55+00:00'
---
