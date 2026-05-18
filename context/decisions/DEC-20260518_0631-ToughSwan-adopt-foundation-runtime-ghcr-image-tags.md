---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260518_0631-ToughSwan-adopt-foundation-runtime-ghcr-image-tags
  created: '2026-05-18T06:31:55+00:00'
  updated: '2026-05-18T07:06:13+00:00'
spec:
  title: Adopt foundation/runtime GHCR image tags for next minor release
  state: accepted
  decision: 'Starting with the next minor aibox release, publish base Debian images
    as foundation/runtime tags: base-debian-foundation-vX.Y.Z for the slow-moving
    OS/tool foundation and base-debian-runtime-vX.Y.Z plus base-debian-runtime-latest
    for the user-facing runtime image. Stop publishing public base-debian-source-<sha>
    tags. Keep CLI compatibility for legacy base-debian-v0.26.x and base-debian-latest
    tags through the transition, but generate new projects against the runtime tag
    family after the cutover.'
  context: The current release-host optimization publishes source-hash marker tags
    such as base-debian-source-<sha>. Those tags leak implementation details into
    GHCR, make the package UI confusing, and made cleanup hazardous because deleting
    versions or source marker entries can leave remaining tags pointing at missing
    child manifests. The owner approved a hard cut to the new tagging system for the
    next minor release, while retaining compatibility for existing legacy tags.
  rationale: Foundation/runtime tags keep the public namespace understandable and
    align tags with user-visible image roles. Source SHA comparison should move into
    image labels and release-script in-situ inspection of recent published tags, avoiding
    public hash tags while preserving the retag-only optimization when image contents
    are unchanged. Splitting foundation from runtime should reduce upload size and
    rebuild time because most releases change only runtime/config layers.
  consequences: Release tooling must inspect recent runtime/foundation manifests and
    labels instead of source marker tags. Dockerfiles/build scripts need a foundation/runtime
    split and OCI source-sha labels. GHCR cleanup must be manifest-aware and must
    not delete shared manifests that still carry retained version tags. Image upload/build
    steps are deferred when the owner is on low-quota network.
  decided_at: '2026-05-18T06:31:55+00:00'
  related_workitems:
  - BACK-20260518_0632-FocusedDaisy-ghcr-foundation-runtime-tags
---
