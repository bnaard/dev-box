---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2237-GrandBear-smarttide-54-per-skill-mcp-drift
  created: '2026-05-09T22:37:03+00:00'
spec:
  title: 'SmartTide #54 — per-skill MCP drift detection implemented (unblocked)'
  body: |
    ## Status

    WorkItem BACK-20260424_0019-SmartTide-github-54-aibox-sync is unblocked and implemented.

    ## Staleness Check (Step 1)

    The original blocker was: "processkit v0.19.2 shipping a manifest contract to track which specs came from which skill."

    Evidence of resolution in installed processkit v0.25.8:
    - Per-skill `mcp/mcp-config.json` files exist for all 44+ processkit skills under `context/skills/processkit/<skill>/mcp/mcp-config.json`. These ARE the per-skill manifest contract the WorkItem required.
    - The aibox compat.rs entry for v0.19.2 confirms: "implements MCP config fingerprint tracking (issue #54) to detect per-skill config drift without version bumps."
    - However, only a coarse whole-tree fingerprint (SHA256 over all files) was implemented in v0.19.2, not the granular per-skill attribution.

    ## Implementation (Step 2)

    Added `detect_per_skill_mcp_config_drift(project_root, merged_mcp_path) -> Vec<PerSkillDrift>` in `cli/src/mcp_registration.rs`.

    Algorithm:
    1. Parse `.mcp.json` into server_name → (command, args) map
    2. Walk `context/skills/processkit/*/mcp/mcp-config.json`
    3. For each server entry: detect MissingFromMerged or EntryMismatch (command/args diff; env excluded)
    4. Return sorted list of PerSkillDrift structs

    Wired into `cmd_sync` (container.rs) — emits a warn line per drifted server before the unconditional regeneration step.

    Five unit tests: clean match, args drift, missing server, pre-sync no-.mcp.json guard, multi-skill drift.

    ## Files Changed

    - `cli/src/mcp_registration.rs` — new function + types + 5 tests
    - `cli/src/container.rs` — call site in cmd_sync fingerprint drift block

    ## Branch / Commit

    Worktree: agent-a78e211b14d1fb6e2 (branch: worktree-agent-a78e211b14d1fb6e2)
    Commit: f1a95c9 "feat(mcp): add per-skill drift detection for .mcp.json reconcile (closes #54)"
  type: fleeting
  state: captured
  review_due: '2026-05-17'
  tags:
  - mcp
  - drift-detection
  - github-54
  - smarttide
  source: agent SmartTide implementation run 2026-05-10
---

## Status

WorkItem BACK-20260424_0019-SmartTide-github-54-aibox-sync is unblocked and implemented.

## Staleness Check (Step 1)

The original blocker was: "processkit v0.19.2 shipping a manifest contract to track which specs came from which skill."

Evidence of resolution in installed processkit v0.25.8:
- Per-skill `mcp/mcp-config.json` files exist for all 44+ processkit skills under `context/skills/processkit/<skill>/mcp/mcp-config.json`. These ARE the per-skill manifest contract the WorkItem required.
- The aibox compat.rs entry for v0.19.2 confirms: "implements MCP config fingerprint tracking (issue #54) to detect per-skill config drift without version bumps."
- However, only a coarse whole-tree fingerprint (SHA256 over all files) was implemented in v0.19.2, not the granular per-skill attribution.

## Implementation (Step 2)

Added `detect_per_skill_mcp_config_drift(project_root, merged_mcp_path) -> Vec<PerSkillDrift>` in `cli/src/mcp_registration.rs`.

Algorithm:
1. Parse `.mcp.json` into server_name → (command, args) map
2. Walk `context/skills/processkit/*/mcp/mcp-config.json`
3. For each server entry: detect MissingFromMerged or EntryMismatch (command/args diff; env excluded)
4. Return sorted list of PerSkillDrift structs

Wired into `cmd_sync` (container.rs) — emits a warn line per drifted server before the unconditional regeneration step.

Five unit tests: clean match, args drift, missing server, pre-sync no-.mcp.json guard, multi-skill drift.

## Files Changed

- `cli/src/mcp_registration.rs` — new function + types + 5 tests
- `cli/src/container.rs` — call site in cmd_sync fingerprint drift block

## Branch / Commit

Worktree: agent-a78e211b14d1fb6e2 (branch: worktree-agent-a78e211b14d1fb6e2)
Commit: f1a95c9 "feat(mcp): add per-skill drift detection for .mcp.json reconcile (closes #54)"
