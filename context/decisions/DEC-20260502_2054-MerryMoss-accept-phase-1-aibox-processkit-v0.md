---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260502_2054-MerryMoss-accept-phase-1-aibox-processkit-v0
  created: '2026-05-02T20:54:41+00:00'
spec:
  title: Accept phase 1 aibox processkit v0.25.0 integration plan
  state: accepted
  decision: 'Phase 1 of the next aibox release will implement the processkit v0.25.0
    integration plan: checkpoint the dirty worktree; bump and install processkit v0.25.0;
    integrate processkit-gateway as a selectable MCP topology; add managed gateway
    runtime support for stdio, daemon plus stdio proxy, and lazy catalog mode where
    appropriate; preserve migration safety for demoted legacy primitives; design the
    optional hard reset path without making it the default; update compatibility docs
    and examples; validate with unit, integration, Docusaurus, processkit gateway,
    pk-doctor, cargo test, clippy, and release gates.'
  context: The owner accepted the previously proposed phase 1 plan and requested it
    be recorded before proposing phase 2. processkit v0.25.0 is available and includes
    processkit-gateway plus v2 contract changes beyond the original SmoothRiver roadmap.
    The release target now includes both processkit v0.25.0 integration and later
    SteadyTiger changes, potentially in two phases.
  rationale: The gateway integration is now concrete and directly addresses the memory/process-count
    issue that blocked reliable aibox devcontainers. Keeping the hard reset optional
    preserves the existing migration flow while providing a path for major context
    refreshes. Separating phase 1 from broader SteadyTiger work reduces release risk
    while still allowing the next release to include all desired changes if phase
    2 is accepted.
  alternatives:
  - option: Implement all processkit v0.25.0 and SteadyTiger work in one unstructured
      pass
    reason_rejected: Too much coupling and release risk; gateway integration and migration
      safety need to land cleanly first.
  - option: Release only the processkit version bump without gateway support
    reason_rejected: Would leave the main user-visible benefit of v0.25.0 unused and
      would not solve eager MCP process pressure.
  - option: Make hard reset the default upgrade path
    reason_rejected: The handover explicitly says the harder reset path should be
      optional; normal migration remains the default.
  consequences: Implementation starts with processkit v0.25.0 and gateway support.
    Full SteadyTiger items such as dual image streams, workspace-manifest schema promotion,
    addon-spec policy enforcement, provider-backend specs, SBOM/provenance, and headless-runner
    release hardening require a separate phase 2 plan and acceptance. The dirty worktree
    must be checkpointed before invasive sync or release work.
  decided_at: '2026-05-02T20:54:41+00:00'
---
