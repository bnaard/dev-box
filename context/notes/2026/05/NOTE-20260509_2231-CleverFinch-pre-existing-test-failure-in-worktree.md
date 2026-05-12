---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2231-CleverFinch-pre-existing-test-failure-in-worktree
  created: '2026-05-09T22:31:39+00:00'
spec:
  title: 'Pre-existing test failure in worktree: standardize_aibox_toml test requires cli/context symlink'
  body: |
    **Observed during TallFrog wasm-bindgen review (2026-05-10)**

    When running `cargo test --bin aibox` from a git worktree (`/workspace/.claude/worktrees/*`), the test `migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape` fails because `skill_catalog_entries_for_comments` uses `std::env::current_dir()` to locate `context/skills/`. In the main workspace, `/workspace/cli/context` is a symlink to `../context`, making the path resolve correctly. In a worktree's `cli/` directory, this symlink is absent.

    **Root cause:** `cli/context -> ../context` symlink is not tracked in git (it's gitignored or hand-created), so worktrees don't have it.

    **Impact:** 1 test fails in all worktree environments. The test passes on main workspace.

    **Trigger for fix:** When setting up worktree-based CI or validating that agent worktrees produce reliable test results. Fix options: (a) track the symlink in git, (b) make the code use a config-driven root path instead of CWD, (c) create the symlink in worktree setup hooks.

    **Related WorkItem:** BACK-20260508_1214-TallFrog-review-wasm-bindgen-updates
  type: fleeting
  state: captured
  review_due: '2026-06-01'
  tags:
  - worktree
  - test-infra
  - pre-existing
  - symlink
  - cli
---

**Observed during TallFrog wasm-bindgen review (2026-05-10)**

When running `cargo test --bin aibox` from a git worktree (`/workspace/.claude/worktrees/*`), the test `migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape` fails because `skill_catalog_entries_for_comments` uses `std::env::current_dir()` to locate `context/skills/`. In the main workspace, `/workspace/cli/context` is a symlink to `../context`, making the path resolve correctly. In a worktree's `cli/` directory, this symlink is absent.

**Root cause:** `cli/context -> ../context` symlink is not tracked in git (it's gitignored or hand-created), so worktrees don't have it.

**Impact:** 1 test fails in all worktree environments. The test passes on main workspace.

**Trigger for fix:** When setting up worktree-based CI or validating that agent worktrees produce reliable test results. Fix options: (a) track the symlink in git, (b) make the code use a config-driven root path instead of CWD, (c) create the symlink in worktree setup hooks.

**Related WorkItem:** BACK-20260508_1214-TallFrog-review-wasm-bindgen-updates
