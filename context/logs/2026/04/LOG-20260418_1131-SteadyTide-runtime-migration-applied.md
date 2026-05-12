---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260418_1131-SteadyTide-runtime-migration-applied
  created: '2026-04-18T11:31:00Z'
spec:
  event_type: migration.applied
  timestamp: '2026-04-18T11:31:00Z'
  actor: claude-opus-4-7
  summary: "Applied MIG-RUNTIME-20260418T090634 \u2014 aibox runtime 0.18.3 -> 0.18.5 (1 new accepted, 1 local kept)"
  related:
  - MIG-RUNTIME-20260418T090634
  details:
    transitioned_from: pending
    transitioned_to: applied
    files_accepted_new: 1
    files_kept_local: 1
    notes: ".aibox-home/.claude.json picked up from upstream (already on disk after sync).\n.aibox-home/.config/git/config retained locally \u2014 owner-personalised file, divergence intentional."
---
