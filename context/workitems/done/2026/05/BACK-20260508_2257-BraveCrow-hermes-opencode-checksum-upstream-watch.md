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
  description: |
    ## Goal

    Watch upstream for SHA-256 checksums or GPG signatures on Hermes (Nous Research) and OpenCode (opencode.ai) release assets, and add verification when available.

    ## Current state (as of v0.25.6)

    Both `addons/ai/ai-hermes.yaml` and `addons/ai/ai-opencode.yaml` already document the upstream gap with detailed `TODO(sec)` blocks:

    - **Hermes** (`addons/ai/ai-hermes.yaml`):
      > "if/when Nous Research adds per-release SHA-256SUMS or GPG signatures, replace the version-pinned download below with a full checksum/GPG verify step"
    - **OpenCode** (`addons/ai/ai-opencode.yaml`):
      > "if/when https://github.com/opencode-ai/opencode/releases begins publishing checksums, add sha256sum verification mirroring the pattern used for [other addons]"

    Current installer posture for both addons is **already the best practical option**: pinned versioned download from GitHub release assets (vs the prior `curl | bash` installer, which was the v0.25.4-era state).

    ## Why this is a watch-upstream item, not implementation

    There is no integrity material to verify against until upstream publishes it. Adding a fake/recorded hash that we compute ourselves provides no security guarantee — it would only catch transport corruption, not malicious release substitution. The honest posture is the version-pinned download we already have.

    ## Trigger to revisit

    - Nous Research adds a `SHA256SUMS` or `*.asc` artifact to a Hermes GitHub release.
    - opencode-ai/opencode adds the same.

    ## Action when triggered

    1. Update the relevant addon yaml's `runtime:` block to include the verify step, mirroring the pattern from a checksumming addon (search the addons tree for `sha256sum -c` examples).
    2. Remove the `TODO(sec)` block.
    3. Bump the addon's `version:` to record the security-posture change.
    4. File a Migration entity if any derived projects need to re-apply.

    ## Why deferred from v0.25.6 (DEC-20260508_2235-CuriousBadger reversal)

    DEC-20260508_2235-CuriousBadger initially listed S1 as a v0.25.6 must-do based on the prior session's handover. The 2026-05-09 scope-pass found that S1 is **not actionable** until upstream changes, and that the existing TODO(sec) blocks are already the correct documentation. Deferred per the v0.25.6 deferred-item scope-pass triage on 2026-05-09.
  completed_at: '2026-05-10T03:25:09+00:00'
---

## Transition note (2026-05-10T03:25:09+00:00)

Watch-only outcome captured in NOTE-20260509_2223-TrueCrane (review_due 2026-06-10). No upstream SHA256SUMS/.asc artifacts available for Hermes or OpenCode; both harnesses also commented out in aibox.toml. Re-evaluate per the Note's review_due trigger; if the gap closes, file a fresh WorkItem.
