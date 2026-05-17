---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260515_1646-HonestHarvest-legacy-concrete-theme-test-guard
  created: '2026-05-15T16:46:06+00:00'
spec:
  title: Guard against legacy concrete theme names in --theme test fixtures
  state: backlog
  type: task
  priority: low
  description: |
    ## Background

    After the v0.26.5 theme-model refactor (clap `--theme` value enum is now `ThemeFamily` only — concrete names like `gruvbox-dark` rejected), the v0.26.5 release pipeline broke 3+ times in succession because tests still passed legacy concrete names:

    - `cli/tests/e2e/config_coverage.rs` asserted on the old `line2-left = ["git", "github", ...]` default
    - `cli/tests/e2e/visual_rendered_starship.rs` passed `--theme=gruvbox-dark`, `--theme=catppuccin-mocha`
    - `cli/tests/e2e/visual.rs::THEME_SIGNATURES` and `cli/tests/e2e/visual_matrix.rs::FULL_MATRIX_THEMES` used concrete names

    Each was fixed reactively (commits b8a01379, c6d06693, 29d22a12). A future theme refactor or anyone adding a new e2e test could repeat the gap silently.

    ## Acceptance criteria

    Add a fast guard that flags legacy concrete theme names in test fixtures BEFORE the release pipeline catches them at runtime. Pick one of:

    - [ ] **Option A (lightweight):** A unit test in `cli/src/themes.rs::tests` (or a new `cli/tests/style.rs`) that walks `cli/tests/`, greps for `--theme",\s*"<concrete>"` patterns where `<concrete>` matches the legacy concrete-name set (gruvbox-dark, ayu-dark, catppuccin-mocha, ...), and fails with a remediation hint pointing at the family form.
    - [ ] **Option B:** A pk-doctor check (under `context/skills/processkit/pk-doctor/scripts/checks/`) that does the same scan and reports a WARN.
    - [ ] **Option C:** A clippy-internal lint impossible without nightly — skip.

    Recommend Option A — fast, no doctor pass needed, fails the regular `cargo test` suite.

    ## Out of scope

    Backcompat for `theme = "ayu-dark"` in aibox.toml (deserializer accepts it; that's intentional and tested). The guard should ONLY flag CLI flag uses (`--theme` arguments) where clap rejects.

    ## Risk

    Low. The guard would have caught all 3 v0.26.5 release breakages.
---
