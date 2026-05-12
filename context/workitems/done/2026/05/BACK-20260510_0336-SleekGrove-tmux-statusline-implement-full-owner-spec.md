---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0336-SleekGrove-tmux-statusline-implement-full-owner-spec
  created: '2026-05-10T03:36:09+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-statusline
    needs-migration: 'true'
  updated: '2026-05-10T07:14:25+00:00'
spec:
  title: "tmux statusline: implement full owner-spec reorganization (line1-right + line2-left + line2-right) \u2014 paired Migration required"
  state: done
  type: bug
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:14:06+00:00'
  completed_at: '2026-05-10T07:14:25+00:00'
---

## Transition note (2026-05-10T07:14:25+00:00)

Implemented and merged in commit 7a094a1 + merge a1a1e9e. Full four-section statusline layout in images/base-debian/config/tmux/tmux.conf; aibox-side cli/src/tmux/status.rs already routed plugin selection so this lit it up end-to-end. Paired Migration MIG-STATUSLINE-20260510T000000 placed in context/migrations/pending/. 31 tmux tests pass. 18 plugins flagged in the Migration body as missing-from-source — file follow-up WorkItems for each.
