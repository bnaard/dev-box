---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260813_1728-OpenDeer-fix-axe-playwright-context-host-probe
  created: '2026-08-13T17:28:04+00:00'
  updated: '2026-08-13T17:28:32+00:00'
spec:
  title: Fix axe Playwright explicit context host probe
  state: done
  type: bug
  priority: critical
  description: The v0.32.1 macOS addon-tools host gate reaches Chromium but axe-core
    rejects browser.newPage(); use browser.newContext() and context.newPage(), close
    the context, validate, and release v0.32.2.
  started_at: '2026-08-13T17:28:08+00:00'
  completed_at: '2026-08-13T17:28:32+00:00'
---

## Transition note (2026-08-13T17:28:08+00:00)

Reproduced from host evidence: axe-core requires an explicit Playwright BrowserContext.


## Transition note (2026-08-13T17:28:32+00:00)

Explicit BrowserContext implementation and regression assertions pass host-gate and release-maintenance contracts.


## Transition note (2026-08-13T17:28:32+00:00)

Fix complete and ready for v0.32.2 release integration.
