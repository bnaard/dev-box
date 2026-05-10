---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260423_2050-EagerStone-extend-aibox-doctor-to
  created: '2026-04-23T20:50:38+00:00'
  updated: '2026-04-26T16:28:37+00:00'
spec:
  title: Extend aibox doctor to detect missing command registrations
  state: done
  type: task
  priority: medium
  description: Add a check to `aibox doctor` (extends BACK-020) that validates every
    installed processkit skill has its command files registered in `.claude/commands/`.
    Should warn if `context/skills/processkit/<skill>/commands/<cmd>.md` exists but
    `.claude/commands/<cmd>.md` is missing. Helps derived projects detect the v0.19.1
    pk-doctor issue before it causes problems.
  started_at: '2026-04-24T00:04:09+00:00'
  completed_at: '2026-04-26T16:28:37+00:00'
---

## Transition note (2026-04-24T00:04:13+00:00)

Implemented command registration detection in aibox doctor. Commit fcd25f2 adds check_command_registrations() that validates all installed skills have their command files registered in .claude/commands/. All tests pass, no clippy warnings.


## Transition note (2026-04-26T16:28:37+00:00)

Superseded by commit 5204969 (v0.21.0). `check_command_registrations` in cli/src/doctor.rs:189 iterates per enabled harness profile and warns when a SKILL.md declares a command but the per-harness target file is missing. Coverage extends beyond `.claude/commands/` to all five scaffolded harnesses. Note: a separate symptom for codex was filed today as BACK-20260426_1627-StrongHawk — files are written but Codex CLI 0.125.0 no longer surfaces them; that's a codex-side regression, not a missing-doctor-check.
