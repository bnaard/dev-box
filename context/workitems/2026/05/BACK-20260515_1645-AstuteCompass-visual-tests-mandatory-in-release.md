---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260515_1645-AstuteCompass-visual-tests-mandatory-in-release
  created: '2026-05-15T16:45:47+00:00'
spec:
  title: Make visual matrix tests + cast-invariants sweep mandatory in cmd_release
  state: backlog
  type: task
  priority: high
  description: |
    ## Background

    aibox v0.26.5 shipped without the new cast-invariants sweep ever running, and with the visual matrix tests still `#[ignore]`'d. The release's `visual` step at scripts/maintain.sh:1347 reads `AIBOX_RELEASE_VISUAL_E2E` which defaults to `skip` — so even the existing release-gated cargo visual tests didn't run. The release log emits the warn `Skipping opt-in visual E2E during release` and continues.

    Meanwhile the new `scripts/test-screencasts.sh themes` Python ANSI cast-invariants sweep (60+ themes, ~5 min, catches I1/I2/I3/I4 regressions like the powerkit GH-separator bug) is wired into `cmd_test_visual` only — never invoked by `cmd_release`.

    ## Acceptance criteria

    - [ ] Un-ignore the 3 tests in `cli/tests/e2e/visual_matrix.rs`:
      - `visual_generated_layouts_render_across_all_themes`
      - `visual_generated_tools_and_harness_windows_render_when_enabled`
      - `visual_yazi_previews_git_symbols_and_optional_plugins_render`
    - [ ] Update / repurpose the `cmd_test_e2e_visual_status/tabs/yazi` wrappers in `scripts/maintain.sh` (drop the `--ignored` flag; either keep the wrappers as convenience entry points or remove and update the `case` dispatcher at line ~1721).
    - [ ] Update `cmd_release`'s `visual` step (around line 1347) so it ALWAYS runs both: (a) the visual matrix cargo tests; (b) `bash scripts/test-screencasts.sh themes`. Either remove the `AIBOX_RELEASE_VISUAL_E2E` env var entirely or change its default to a value that runs everything (with an emergency `AIBOX_RELEASE_SKIP_VISUAL=1` opt-out that emits a loud warn).
    - [ ] Update the canonical release process documentation (likely under `context/notes/` or `context/work-instructions/`; look for a release-process note or `AGENTS.md` "Pull requests" section) to document the new mandatory gate.
    - [ ] Cargo build / clippy / test all clean after the changes.
    - [ ] Run the full mandated suite end-to-end at least once and confirm every theme passes the cast-invariants check.

    ## Operational notes

    The previous agent dispatch attempt was rejected because the brief was too large and the user wanted runtime/monitoring options first. Next time:
    - Estimated runtime: visual matrix ~10-15 min (narrow) + bash sweep ~5-7 min = ~15-25 min total.
    - The bash sweep already emits per-theme PASS/FAIL via `validate_cast` + `assert_cast_visible_status_text` — verify that's granular enough or add per-theme one-liners.
    - The visual matrix uses `log_visual_progress` per case.
    - Consider loading the `Monitor` tool to stream stdout-line notifications so the user sees per-theme progress live during a backgrounded run.

    ## Risk

    If the bash sweep finds a regression on themes other than ayu-dark, fixing it may delay future releases. Run the sweep once locally before committing to making it mandatory, so any baseline failures get triaged first.
---
