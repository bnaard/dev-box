---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260724_1914-ActiveCoral-define-v1-typed-orchestration-config
  created: '2026-07-24T19:14:42+00:00'
  updated: '2026-07-25T09:07:04+00:00'
spec:
  title: Define v1 typed orchestration configuration
  state: done
  type: story
  priority: high
  description: 'Extend aibox configuration with typed v1 image, fleet, deployment
    target, deployment, and connection intent. Separate semantic defaults from backend
    rendering defaults, preserve v0 compatibility, reject invalid backend configuration
    before runtime discovery, and add parsing/validation fixtures and tests. Refs
    GitHub #179 and ART-20260724_1653-FairHare-draft-aibox-v1-image-and-deployment.'
  parent: BACK-20260724_1843-PatientLynx-implement-aibox-v1-orchestration
  scope: v1.x
  started_at: '2026-07-24T19:14:50+00:00'
  completed_at: '2026-07-25T09:07:04+00:00'
---

## Transition note (2026-07-24T19:14:50+00:00)

Starting in an isolated v1.x worktree from pushed branch agent/v1-m0-m1-foundation.


## Transition note (2026-07-24T19:25:48+00:00)

Implemented and pushed on origin/agent/v1-m2-config-compiler. Commits 2aea2e2b and integration adapter 62bec040. Added opt-in typed orchestration config, explicit owner identity, backend-compatible validation, immutable image and credential-reference checks, and v0-default preservation.


## Transition note (2026-07-25T09:07:04+00:00)

Validated and merged into v1.x-dev through PR #183.
