---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260423_2050-LuckyRabbit-aibox-sync-should-auto
  created: '2026-04-23T20:50:41+00:00'
  updated: '2026-04-26T16:28:31+00:00'
spec:
  title: aibox sync should auto-generate missing command registrations
  state: done
  type: task
  priority: medium
  description: Enhance `aibox sync` to auto-generate missing command files. When syncing, scan all installed skills' `SKILL.md` files for `commands:` declarations and auto-create `.claude/commands/<cmd>.md` files from the source definitions in `context/skills/*/commands/` if they don't exist. This provides transparent remediation for incomplete skill installations like pk-doctor in v0.19.1, so derived projects don't need manual intervention.
  started_at: '2026-04-24T00:02:31+00:00'
  completed_at: '2026-04-26T16:28:31+00:00'
---

## Transition note (2026-04-24T00:02:37+00:00)

Ready for review: Implemented auto-generation of missing command adapters in sync_claude_commands. Commit e9623b7 adds SkillCommand struct and generate_missing_command_files function. All unit tests pass, no clippy warnings. GitHub issue projectious-work/processkit#10 filed for upstream fix.


## Transition note (2026-04-26T16:28:31+00:00)

Superseded by commit 5204969 (v0.21.0) feat(harness): scaffold processkit slash commands beyond Claude Code. `sync_harness_commands` in cli/src/harness_commands.rs:159 auto-generates command files for every enabled harness during sync; covers Claude (.claude/commands/), Codex (.aibox-home/.codex/prompts/), Cursor, Gemini, and OpenCode. Original v0.19.1 pk-doctor case was the motivating example and is verified fixed.
