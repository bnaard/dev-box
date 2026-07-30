---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260518_0632-FocusedDaisy-ghcr-foundation-runtime-tags
  created: '2026-05-18T06:32:58+00:00'
  labels:
    decision: DEC-20260518_0631-ToughSwan-adopt-foundation-runtime-ghcr-image-tags
    defer_image_uploads: true
  updated: '2026-07-28T05:56:21+00:00'
spec:
  title: Implement GHCR foundation/runtime image tagging redesign
  state: done
  type: task
  priority: high
  description: Implement the approved next-minor aibox image publishing redesign.
    Stop publishing public base-debian-source-<sha> tags; publish base-debian-foundation-vX.Y.Z
    and base-debian-runtime-vX.Y.Z/base-debian-runtime-latest; keep CLI compatibility
    for legacy base-debian-v0.26.x/base-debian-latest; move source-sha comparison
    into image labels and release-host in-situ inspection of recent tags; add manifest-aware
    cleanup planning. Do not build or upload images while the owner is on slow/low-quota
    network.
  started_at: '2026-05-18T06:38:32+00:00'
  completed_at: '2026-07-28T05:56:21+00:00'
---

## Transition note (2026-05-18T06:38:32+00:00)

Implementation started after owner approval. Image builds/uploads are explicitly deferred due to low-quota network.


## Transition note (2026-07-28T05:56:21+00:00)

Foundation/runtime tag model, source labels, in-situ reuse inspection, compatibility, cleanup planning, and docs are implemented; v0.28.16 public tags were verified.


## Transition note (2026-07-28T05:56:21+00:00)

Acceptance evidence reconciled; no remaining implementation gap.
