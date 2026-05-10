---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0325-LivelyFinch-cli-src-migration-rs-standardize-aibox
  created: '2026-05-10T03:25:26+00:00'
  labels:
    version: v0.25.7-followup
    area: test-infra
    blocks: release-audit
spec:
  title: 'cli/src/migration.rs: standardize_aibox_toml test fails in worktrees due
    to missing cli/context symlink'
  state: backlog
  type: bug
  priority: high
  description: |
    ## Symptom

    `cargo test --bin aibox` produces 1 failure when run in a git worktree: `migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape`. Confirmed pre-existing (≥4 v0.25.7 sprint agents independently observed it: TallFrog/CleverFinch, GentleSeal/CoolMeadow, SnappyThorn, PluckyEagle).

    Passes when run from `/workspace/cli` (the main checkout). Worktrees lack the symlink.

    ## Root cause

    `cli/context` is a symlink to `../context` — used at unit-test time by `migration.rs:2372`'s skill-catalog lookup (CWD-relative). The symlink is **not tracked in git**, so worktrees created via `git worktree add` don't inherit it, and the catalog read returns no entries. The test then sees `actor-profile` get the comment `"custom skill override"` instead of the expected `"processkit;"`.

    ## Fix candidates

    1. **Track the symlink in git** (simplest): `git add cli/context` after teaching git to track symlinks (it does by default; the symlink may have been gitignored). Verify `.gitignore` and commit.
    2. **Mock the catalog lookup** in the test (more robust): replace the CWD-relative read with an injected fixture. Decouples the test from filesystem layout.
    3. **Add a test setup hook** that symlinks at test-init time if missing.

    Recommend option 2 (mock) — the test should not depend on FS state outside its fixture.

    ## Acceptance

    - `cargo test --bin aibox` passes from any worktree as well as `/workspace/cli`.
    - No regression to the `standardize_aibox_toml` semantic (canonical shape rewrite still works).

    ## Refs

    - NOTE-20260509_2231-CleverFinch (TallFrog observation)
    - NOTE-20260509_2231-CoolMeadow (GentleSeal root-cause analysis)
    - Files: `cli/src/migration.rs:2372`, `cli/context` (symlink target `../context`)
---
