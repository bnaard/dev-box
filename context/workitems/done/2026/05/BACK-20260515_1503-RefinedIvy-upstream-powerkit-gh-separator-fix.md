---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260515_1503-RefinedIvy-upstream-powerkit-gh-separator-fix
  created: '2026-05-15T15:03:20+00:00'
  updated: '2026-08-21T19:58:50+00:00'
spec:
  title: Upstream the tmux-powerkit GH-segment separator fix
  state: cancelled
  type: task
  priority: low
  description: |
    ## Background

    We carry a local patch to `tmux-powerkit`'s `src/renderer/segment_builder.sh` that adds a `bg=<surface>` space cell between adjacent segment separators when `@powerkit_elements_spacing = "both"` is in effect. Without this fix, the GH segment renders as "two arrows on default bg" — visible on light terminals and reproducible by the cast-invariants helper (I2 fail at row 1 col 18 on ayu-dark recording).

    The patch is currently applied as a `sed`/`patch` step in `images/base-debian/Dockerfile` after `install_plugin fabioluciano/tmux-powerkit ${TMUX_POWERKIT_REF}` runs. This means every `aibox` base image rebuild re-patches the upstream tarball.

    ## Goal

    Decide whether to upstream the fix to https://github.com/fabioluciano/tmux-powerkit. If accepted, bump `TMUX_POWERKIT_REF` to a tag/SHA that includes the fix and drop the `sed` patch from the Dockerfile.

    ## Acceptance criteria

    - [ ] Reproduce the fix as a clean diff against current HEAD of the upstream repo.
    - [ ] Confirm the bug is reproducible upstream (might be a config-specific edge case in our setup).
    - [ ] Open an issue or PR on the upstream repo with a minimal reproducer cast.
    - [ ] Once merged, bump `TMUX_POWERKIT_REF` in `images/base-debian/Dockerfile` and remove the local sed patch step.
    - [ ] Re-record `docs-site/static/asciinema/themes/*.cast` to confirm I2 still passes with the upstream fix.

    ## Priority rationale

    Low — the local patch works fine and re-applies on every image rebuild. The cost is just one extra `RUN sed` line in the Dockerfile and a known-divergence note. If upstream is responsive, escalate; otherwise the patch can live indefinitely.

    ## References

    - Diagnostic commit: `9fa97fb7` — "feat: powerkit-aware theme recordings + Docusaurus gallery"
    - Patch commit: TBD (this turn)
    - Upstream pinned ref: `139be6bbd57dbedfc6c534e72a440147ad0ab4d4` (in `images/base-debian/Dockerfile`)
    - Upstream repo: https://github.com/fabioluciano/tmux-powerkit
    - Live patched file: `/usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/renderer/segment_builder.sh` (lines 799-801)
  completed_at: '2026-08-21T19:58:50+00:00'
---

## Transition note (2026-08-21T19:58:50+00:00)

Closed as no longer relevant by owner direction on 2026-08-21.
