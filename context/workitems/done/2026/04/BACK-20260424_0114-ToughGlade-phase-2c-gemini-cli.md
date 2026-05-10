---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0114-ToughGlade-phase-2c-gemini-cli
  created: '2026-04-24T01:14:32+00:00'
  updated: '2026-04-24T07:47:47+00:00'
spec:
  title: 'Phase 2c: Gemini CLI + GitHub Copilot generators'
  state: done
  type: task
  priority: high
  description: |
    **Scope:** Implement policy-based generators for Gemini CLI and GitHub Copilot.

    **Deliverables:**
    1. `generate_gemini_permissions(config: &McpConfig) -> Result<()>`
       - Generate dual `includeTools[]` (allowlist) and `excludeTools[]` (blocklist)
       - Apply intersection semantics: a tool is allowed if in includeTools AND not in excludeTools
    2. `generate_github_copilot_permissions(config: &McpConfig) -> Result<()>`
       - Generate CLI flags: `--allow-tool`, `--deny-tool`
       - Document enterprise registry policy integration

    **Key Difference:** Both support dual allow/deny; Gemini uses intersection, Copilot uses command-line flags.

    **Estimated Tokens:** ~2.5K per harness (dual list logic, flag generation)
    **Can run in parallel:** with Phase 2a, 2b, 2d
  parent: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  blocked_by:
  - BACK-20260424_0114-JollyStream-phase-1-core-mcp
  started_at: '2026-04-24T07:47:46+00:00'
  completed_at: '2026-04-24T07:47:47+00:00'
---

## Transition note (2026-04-24T07:47:46+00:00)

Gemini and GitHub Copilot generators implemented with dual allowlist/denylist support.


## Transition note (2026-04-24T07:47:47+00:00)

Phase 2c complete. All Phase 2 generators now ready.
