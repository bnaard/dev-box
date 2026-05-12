---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260426_1627-StrongHawk-codex-cli-0-125
  created: '2026-04-26T16:27:28+00:00'
  updated: '2026-04-29T10:09:10+00:00'
spec:
  title: 'Codex CLI 0.125.0: /pk-* slash commands not surfaced from ~/.codex/prompts/'
  state: done
  type: bug
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-04-26T16:33:44+00:00'
  completed_at: '2026-04-29T10:09:10+00:00'
---

## Transition note (2026-04-26T16:33:44+00:00)

Starting investigation: research what changed in Codex 0.125.0's prompt-loading mechanism, then determine the fix for cli/src/harness_commands.rs Codex profile.


## Transition note (2026-04-29T10:09:04+00:00)

Reconciliation review: Codex pk command scaffolding now targets .agents/skills/<name>/SKILL.md with legacy prompt cleanup; decision DEC-20260426_1636-MightySky records the integration surface.


## Transition note (2026-04-29T10:09:10+00:00)

Closed during 2026-04-29 reconciliation after confirming Codex Skills scaffolding and doctor/perimeter updates are present in the working tree.
