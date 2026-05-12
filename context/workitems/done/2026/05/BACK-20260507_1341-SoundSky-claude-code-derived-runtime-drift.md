---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1341-SoundSky-claude-code-derived-runtime-drift
  created: '2026-05-07T13:41:08+00:00'
  labels:
    area: claude-code
    runtime: derived-project
    source: 2026-05-07-diagnostics
  updated: '2026-05-07T13:59:31+00:00'
spec:
  title: Fix Claude Code MCP auth, install-path, and permission drift in derived aibox runtimes
  state: done
  type: bug
  priority: high
  description: 'Derived project on aibox 0.24.2 showed Claude Code v2.1.132 reporting MCP auth/system diagnostic problems while aibox generated only processkit-gateway in .mcp.json. Evidence: Claude status said 1 MCP connected and 3 need auth; installMethod native but claude not found at /home/aibox/.local/bin/claude despite command -v resolving /usr/local/bin/claude; leftover npm global installation at /usr/bin/claude; .claude/settings.json contains many granular processkit server permissions while .mcp.json only declares processkit-gateway; Zellij permission cache missing at .aibox-home/.cache/zellij/permissions.kdl. Acceptance: doctor or a dedicated diagnosis command captures these facts, generator output is reconciled, and a fresh derived project no longer reports Claude MCP auth/install drift after apply/recreate.'
  scope: runtime
  started_at: '2026-05-07T13:59:20+00:00'
  completed_at: '2026-05-07T13:59:31+00:00'
---

## Transition note (2026-05-07T13:59:20+00:00)

Implementation started in aibox source: Claude MCP gateway drift pruning, Claude home-bin shim, and doctor drift checks.


## Transition note (2026-05-07T13:59:27+00:00)

Implementation complete; focused regressions, cargo test, clippy, and standalone aibox-status compile passed.


## Transition note (2026-05-07T13:59:31+00:00)

Accepted for this implementation slice: Claude Code MCP gateway drift is pruned from generated settings, Claude home-bin shim is seeded, doctor reports drift, and validation passed.
