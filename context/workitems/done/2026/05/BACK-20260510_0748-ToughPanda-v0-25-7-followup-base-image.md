---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0748-ToughPanda-v0-25-7-followup-base-image
  created: '2026-05-10T07:48:12+00:00'
  labels:
    version: v0.25.7-followup
    area: base-image
    needs-decision: 'true'
  updated: '2026-05-10T08:09:03+00:00'
spec:
  title: 'v0.25.7-followup: base image install steps for k9s / btop / lazydocker'
  state: cancelled
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  completed_at: '2026-05-10T08:09:03+00:00'
---

## Transition note (2026-05-10T08:09:03+00:00)

Obsolete: the underlying speculative tool additions (k9s/btop/lazydocker) are being reverted in a follow-up WorkItem. Once those tools are not shipped at all, the base-image install decision becomes moot. When/if specific tools are explicitly requested in the future, they'll get their own WorkItem with a per-tool install decision.
