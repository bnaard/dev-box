---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-TallBear-release-audit-stale-layout-tests
  created: '2026-05-09T13:16:56+00:00'
  updated: '2026-05-09T22:19:05+00:00'
spec:
  title: 'release-audit: add stale-test grep sweep for hardcoded layout/window-set
    assumptions'
  state: done
  type: task
  priority: medium
  description: |
    ## Motivation
    Stale tests bit us twice during the v0.25.5→v0.25.6 cut: `visual_kb_yazi_e` in Phase 1 and `release-runtime-smoke` window-set probe in Phase 2 (commit d970e57: "fix(release-smoke): probe expects ai+shell+[git], not stale 'editor' window"). Pattern: a test asserts a hardcoded layout name, window count, pane count, or window-set that doesn't survive `tmux/layouts.rs` edits.

    ## Proposal
    New release-audit check (lives in `pk-release-audit` skill or its CLI counterpart) that:
    1. Renders the current set of layout scripts via `tmux_layout_script` for every `(layout, harness-set)` combination.
    2. Extracts all `-n <window-name>` window names and pane counts.
    3. Greps `cli/tests/**/*.rs` and `scripts/release-smoke/**/*.sh` for window-name / pane-count assertions.
    4. Flags any test asserting a name/count not present in the rendered set.

    ## Acceptance criteria
    - Check is invocable as `pk-release-audit` step (or `aibox doctor --check stale-layout-tests`).
    - Detects `-n editor` in tests when `editor` is not in any rendered layout (regression-replays the v0.25.6 incident).
    - Detects pane-count mismatches.
    - Runs in <2s on the current test corpus (no full builds).

    ## Notes
    - Should be parameterized: tests legitimately may assert "must NOT contain editor" — check distinguishes positive vs negative assertions.
    - Likely overlaps with Bug 1 (multi-harness layouts) — that change will produce more `(layout, harness-set)` combinations to audit; this check should fan out across all of them.

    ## References
    - Commit d970e57 — `fix(release-smoke): probe expects ai+shell+[git], not stale 'editor' window`
    - Phase 1 incident: `visual_kb_yazi_e` test stale after `BR-VIM-HARDCUT` (DEC-20260508_1604-LuckySeal)
    - WorkItem context/workitems/BACK-20260508_2303-GentleFern (BR-CLEANUP-ARCH item 6 — Variant 3 Migration emission; complements this one)
  started_at: '2026-05-09T22:18:28+00:00'
  completed_at: '2026-05-09T22:19:05+00:00'
---

## Transition note (2026-05-09T22:19:05+00:00)

Implemented and merged in commit 62889df + merge e427e62. Ships project-local prototype scripts/release-audit-stale-tests.py with 32-hit baseline plus propose-only upstream patch.
