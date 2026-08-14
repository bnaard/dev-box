# Roadmap

LLMS index: [llms.txt](/aibox/v0.x/v0.32.3/llms.txt)

---

# Roadmap

This page outlines planned features and improvements for aibox. The internal
source of truth is the processkit work item index under `context/`; this page is
the public-facing summary.

## Current Focus

Current work is focused on making long-running AI workspaces cheaper and more
predictable:

- explicit Compose project and image names for each aibox project
- optional tool bundles so idle containers do not carry unnecessary CLIs
- suspended non-focused tmux panes
- runtime resource snapshots and doctor thresholds
- init-reaper support for orphaned helper processes
- processkit MCP gateway adoption and daemon validation in real projects

## Planned — Near Term

### processkit Gateway Follow-Through

Keep validating the gateway-mode MCP defaults in downstream projects and tune
daemon guidance as more host/container runtime combinations are exercised.

### Runtime Diagnostics

Extend resource reporting with zombie process counts and clearer actionable
doctor output for memory and process pressure.

### Documentation and Onboarding

Keep the public docs aligned with the current verb/resource CLI grammar,
processkit boundary, addon selection model, and runtime operations.

## Planned — Medium Term

### Remote Development Boundary

Clarify what belongs in aibox versus a dedicated infrastructure/deployer CLI
for remote and cloud-hosted workspaces.

### External Addon and Skill Sources

Broaden source support while preserving pinned, reproducible installs.

### Skill Evaluation Support

Support repeatable checks for installed processkit skills and project-specific
customizations.

## Planned — Long Term

### Multi-Service Workspaces

Improve first-class support for project sidecars and test companions without
turning aibox into a production orchestrator.

### Signed Images and Supply Chain

Add stronger supply-chain verification for published images and release assets.

### Richer Runtime UI

Move beyond shell status lines toward richer tmux-native status integration
when the additional runtime coupling is worth the complexity.
