---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260813_1609-NimbleOtter-fix-chromium-host-probe
  created: '2026-08-13T16:09:41+00:00'
  updated: '2026-08-13T16:10:45+00:00'
spec:
  title: Fix full-Chromium release-host probe and release v0.32.1
  state: done
  type: bug
  priority: high
  description: The v0.32.0 addon-tools host gate installed Playwright full Chromium
    with --no-shell but launched Playwright's default headless-shell executable. Launch
    channel chromium instead, add regression coverage, and publish v0.32.1; v0.32.0
    remains immutable and host-incomplete.
  started_at: '2026-08-13T16:09:47+00:00'
  completed_at: '2026-08-13T16:10:45+00:00'
---

## Transition note (2026-08-13T16:09:47+00:00)

Owner supplied v0.32.0 release-host failure evidence; root cause identified as headless-shell/full-Chromium launch mismatch.


## Transition note (2026-08-13T16:10:41+00:00)

Source probe now explicitly launches channel chromium; release-host contract, focused addon test, formatting, and diff checks pass.


## Transition note (2026-08-13T16:10:45+00:00)

Fix accepted by test evidence and prepared for immutable v0.32.1 patch publication.
