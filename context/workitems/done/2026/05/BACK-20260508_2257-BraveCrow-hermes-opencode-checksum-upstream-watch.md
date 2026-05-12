---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2257-BraveCrow-hermes-opencode-checksum-upstream-watch
  created: '2026-05-08T22:57:28+00:00'
  labels:
    track: security
    release: v0.25.7
    deferred_from: v0.25.6 / DEC-20260508_2235-CuriousBadger
    kind: watch-upstream
  updated: '2026-05-10T03:25:09+00:00'
spec:
  title: 'v0.25.7 watch: Hermes / OpenCode addon checksum upstream gap'
  state: cancelled
  type: task
  priority: low
  description: Migrated historical description; see git history for pre-migration full text.
  completed_at: '2026-05-10T03:25:09+00:00'
---

## Transition note (2026-05-10T03:25:09+00:00)

Watch-only outcome captured in NOTE-20260509_2223-TrueCrane (review_due 2026-06-10). No upstream SHA256SUMS/.asc artifacts available for Hermes or OpenCode; both harnesses also commented out in aibox.toml. Re-evaluate per the Note's review_due trigger; if the gap closes, file a fresh WorkItem.
