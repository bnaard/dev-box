---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0019-SmartTide-github-54-aibox-sync
  created: '2026-04-24T00:19:50+00:00'
  updated: '2026-05-10T03:24:54+00:00'
spec:
  title: 'GitHub #54: aibox sync reconcile .mcp.json on per-skill-config drift'
  state: done
  type: task
  priority: medium
  description: |
    Implement logic to detect and reconcile drift between per-skill mcp-config.json files and the merged .mcp.json at project root. This handles the case where a user updates a skill's mcp-config.json directly (bypassing processkit update), and aibox sync should re-merge to pick up the changes.

    Depends on processkit v0.19.2 shipping a manifest contract to track which specs came from which skill. Currently blocked waiting for upstream manifest.

    Related: GitHub issue #54
  started_at: '2026-05-09T22:37:13+00:00'
  completed_at: '2026-05-10T03:24:54+00:00'
---

## Transition note (2026-05-09T22:37:13+00:00)

Unblocked: processkit v0.25.8 ships per-skill mcp-config.json files (the manifest contract the WorkItem required). Implementing detect_per_skill_mcp_config_drift() in worktree agent-a78e211b14d1fb6e2.


## Transition note (2026-05-09T22:37:20+00:00)

Implementation complete. Commit f1a95c9 in worktree agent-a78e211b14d1fb6e2. Ready for merge to v0.25.7 release branch. Note: NOTE-20260509_2237-GrandBear-smarttide-54-per-skill-mcp-drift.


## Transition note (2026-05-10T03:24:54+00:00)

UNBLOCKED (processkit v0.25.8 ships per-skill mcp/mcp-config.json contract). Implemented and merged in commit f1a95c9 + merge 79f5c5c. New cli/src/mcp_registration.rs::detect_per_skill_mcp_config_drift (+394 LOC) wired in cmd_sync; 5 new drift-detection tests pass. Closes GitHub #54.
