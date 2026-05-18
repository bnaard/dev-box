---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260518_0807-PromptLantern-finish-ghcr-image-tag-rollout
  created: '2026-05-18T08:07:10+00:00'
  labels:
    area: release
    component: ghcr
    release_blocker: true
    before_next_minor: true
    related_workitem: BACK-20260518_0632-FocusedDaisy-ghcr-foundation-runtime-tags
  updated: '2026-05-18T14:41:35+00:00'
spec:
  title: Finish GHCR image-tag rollout before next minor release
  state: done
  type: task
  priority: high
  description: |
    High-priority release-blocker follow-up for the deferred parts of the foundation/runtime image-tag redesign. Must be resolved before cutting the next minor release.

    Remaining items:
    - Run a host-side dry run of `./scripts/maintain.sh ghcr-prune-source-tags` and review source-hash-only vs mixed GHCR package versions.
    - Execute GHCR cleanup only for source-hash-only package versions once network/quota is acceptable; do not delete mixed versions that also carry non-source tags.
    - Build and push the new image family on the host for the next minor release: `base-debian-foundation-vX.Y.Z`, `base-debian-runtime-vX.Y.Z`, and `base-debian-runtime-latest`.
    - Verify no public `base-debian-source-<sha>` tags are created by the release flow.
    - Verify legacy CLI compatibility remains intact for `base-debian-v0.26.x` / `base-debian-latest` until the cutoff release.
    - Confirm release docs and smoke checks reference the new foundation/runtime tags before publishing the next minor release.
  parent: BACK-20260518_0632-FocusedDaisy-ghcr-foundation-runtime-tags
  started_at: '2026-05-18T12:27:59+00:00'
  completed_at: '2026-05-18T14:41:35+00:00'
---

## Transition note (2026-05-18T12:27:59+00:00)

Finalizing the deferred GHCR image-tag rollout items: validate release scripts, inspect GHCR cleanup state, and determine whether host-side image build/push can be completed from this environment.


## Transition note (2026-05-18T12:41:57+00:00)

Source-side rollout checks are complete: release script no longer publishes source-hash tags, new foundation/runtime tags are wired and tested, docs build passes, and ghcr-prune-source-tags now has safe dry-run/delete logic with clear read:packages/delete:packages diagnostics. Remaining live operations are blocked in this container: Docker/Podman are unavailable, and the active GitHub token returns HTTP 403 for GHCR package listing because it lacks read:packages. To finish on host: use a token with read:packages for dry-run and read:packages+delete:packages for --execute, then run host image build/push for the next minor release.


## Transition note (2026-05-18T14:40:50+00:00)

Host-side blocker cleared: owner ran `./scripts/maintain.sh ghcr-prune-source-tags --repair-mixed --execute`; GHCR cleanup now reports no source-hash package versions and no mixed-version warnings.


## Transition note (2026-05-18T14:41:26+00:00)

Implementation and host cleanup complete; moving to review before closure per workitem state machine.


## Transition note (2026-05-18T14:41:35+00:00)

Completed. Host-side GHCR cleanup evidence: `./scripts/maintain.sh ghcr-prune-source-tags --repair-mixed --execute` scanned GHCR and reported `No source-hash-only GHCR package versions found`, with no mixed-version warnings. Source-side release flow, docs, and tests were already validated. The next minor release can publish the new foundation/runtime tags through the normal release-host flow.
