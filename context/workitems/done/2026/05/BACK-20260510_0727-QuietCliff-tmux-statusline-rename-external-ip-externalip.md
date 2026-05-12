---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0727-QuietCliff-tmux-statusline-rename-external-ip-externalip
  created: '2026-05-10T07:27:54+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-statusline
  updated: '2026-05-10T07:40:02+00:00'
spec:
  title: "tmux statusline: rename external_ip \u2192 externalip in line1-right config to match upstream plugin filename"
  state: done
  type: bug
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:39:44+00:00'
  completed_at: '2026-05-10T07:40:02+00:00'
---

## Transition note (2026-05-10T07:40:02+00:00)

Implemented and merged in commit dc3dd36 + merge 08954fc. Renamed external_ip → externalip in tmux.conf @powerkit_plugins / @powerkit_line1_right, cli/src/tmux/status.rs LINE1_RIGHT_ORDER + 4 test strings, cli/src/doctor.rs doc + required array. Migration body cleaned up — replaced misleading 'skipped' section with verified facts. 13 plugin-name hits → 0; 5 remaining external_ip occurrences are Rust struct fields (TOML key 'external-ip') correctly preserved. 878/879 tests pass (1 pre-existing LivelyFinch).
