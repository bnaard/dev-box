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
  description: |
    ## Background

    GrandDaisy (commit `15de96b`, branch `v0.25.7/layouts-trueclover-and-tools`) generalized the tools-as-windows pattern: `addons/tools/monitoring.yaml` declares `btop` (apt) + `lazydocker` (GitHub releases); `addons/tools/kubernetes.yaml` already declares `k9s`. Each addon's Dockerfile stanza will pull in the tool on next `aibox apply` for projects that opt-in.

    But the **pre-built base image** (`images/base-debian/`) doesn't yet ship these binaries. Projects relying on the prebuilt image won't have k9s/btop/lazydocker available even when their addon is enabled — they'd need a base image rebuild.

    ## Goal

    Decide whether to install k9s/btop/lazydocker into the base image by default (since they're addon-driven and small), or keep them addon-only (rebuild required if enabled).

    ## Implementation candidates

    1. **Default-on in base image:** add the install steps to `images/base-debian/Dockerfile`. Cost: image size grows by ~50-100 MB. Benefit: `aibox apply` doesn't trigger an image rebuild for users enabling these.
    2. **Addon-only:** leave them out of the base image. Cost: enabling the addon triggers a rebuild. Benefit: smaller default image; users who don't use these tools don't pay the size cost.
    3. **Hybrid:** install the small ones (`btop` ~5MB, `lazydocker` ~10MB) by default; keep `k9s` (~50MB) addon-only.

    Recommendation: option 3 (hybrid) keeps the default image lean while making the most-likely-used tools available out of the box.

    ## Acceptance

    - A DecisionRecord captures the chosen option.
    - If options 1 or 3: Dockerfile gets the install steps; image version bumps; `aibox.lock` updates the digest pin; `aibox apply` smoke-test confirms the tools are accessible.
    - If option 2: documentation update under `docs-site/docs/addons/` clarifies that enabling these addons triggers a rebuild.

    ## Refs

    - BACK-20260510_0726-GrandDaisy (predecessor — the addon yamls)
    - DEC-20260510_0346-TrueClover (sibling, accepted; layout context)
    - Files: `images/base-debian/Dockerfile`, `addons/tools/{monitoring,kubernetes}.yaml`
  completed_at: '2026-05-10T08:09:03+00:00'
---

## Transition note (2026-05-10T08:09:03+00:00)

Obsolete: the underlying speculative tool additions (k9s/btop/lazydocker) are being reverted in a follow-up WorkItem. Once those tools are not shipped at all, the base-image install decision becomes moot. When/if specific tools are explicitly requested in the future, they'll get their own WorkItem with a per-tool install decision.
