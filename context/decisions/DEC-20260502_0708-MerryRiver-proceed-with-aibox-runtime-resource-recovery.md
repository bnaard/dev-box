---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260502_0708-MerryRiver-proceed-with-aibox-runtime-resource-recovery
  created: '2026-05-02T07:08:58+00:00'
spec:
  title: Proceed with aibox runtime/resource recovery plan before processkit-gated
    roadmap work
  state: accepted
  decision: 'Implement the approved aibox plan in phases: first fix immediate runtime/layout/build/OrbStack/resource
    issues in aibox, then optionalize selected base tools, then harden preview roadmap
    deliverables, while leaving canonical Artifact emission and MCP gateway integration
    gated on processkit delivery.'
  context: The user approved the implementation plan after reports that Codex-only
    projects can still start Claude due to stale runtime layouts, aibox apply lacks
    --no-cache, OrbStack groups projects under devcontainer, and running devcontainers
    have high process/memory pressure. The user also clarified that processkit is
    already working on the MCP server gateway.
  rationale: The immediate fixes are within aibox's ownership boundary and reduce
    operational pain now. The largest process-count reduction depends on processkit's
    gateway work, so aibox should prepare the integration surface but not duplicate
    gateway semantics. Canonical roadmap Artifact emission remains gated on processkit
    schemas/kinds.
  alternatives:
  - option: Wait for processkit before changing aibox
    rejected_because: Does not address current broken startup, no-cache, OrbStack,
      or resource visibility issues.
  - option: Implement an aibox-owned MCP gateway now
    rejected_because: Processkit is already working on the gateway and owns MCP server
      semantics; duplicating it would create competing implementations.
  - option: Force-overwrite runtime layouts on every up/apply
    rejected_because: Would destroy legitimate user edits under .aibox-home; provenance-based
      updates preserve the edit-in-place contract.
  consequences: aibox work can proceed without waiting for processkit on runtime provenance,
    CLI flags, compose identity, zellij lazy startup, and diagnostics. Gateway and
    canonical Artifact work will remain explicit follow-up work after processkit delivery.
  decided_at: '2026-05-02T07:08:58+00:00'
---
