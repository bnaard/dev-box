---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2231-CoolMeadow-pre-existing-test-failure-migration-tests
  created: '2026-05-09T22:31:45+00:00'
spec:
  title: 'Pre-existing test failure: migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape'
  body: |
    ## Pre-existing test failure discovered during GentleSeal rust crate update pass

    **Branch:** v0.25.7/gentleseal-rust-crate-updates
    **Discovered:** 2026-05-10
    **WorkItem:** BACK-20260508_0629-GentleSeal-defer-rust-crate-updates

    ### Failure

    `migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape` panics at `cli/src/migration.rs:2372`:

    ```
    assertion failed: after.contains("    \"actor-profile\", # processkit;")
    ```

    ### Root cause

    `skill_catalog_entries_for_comments()` resolves skill catalog entries from `current_dir().join("context/skills")`. When cargo runs unit tests, `current_dir()` is `cli/`, so it looks for `cli/context/skills/` which doesn't exist. The actor-profile skill entry is therefore not found in the catalog, causing `render_skill_array` to fall back to `"custom skill override"` instead of the expected `"processkit; ..."` comment format.

    The test assertion `after.contains("    \"actor-profile\", # processkit;")` therefore fails.

    ### Not caused by

    This failure is **not** caused by the Cargo.lock patch updates applied in this GentleSeal pass. The Cargo.lock diff is limited to transitive patch bumps (js-sys, wasm-bindgen family, cc, filetime, hashbrown). The `migration.rs` and `container.rs` files are untouched.

    ### Likely cause

    A recent v0.25.7 merge (probably the skills catalog rendering feature or a test refactor) introduced this test without accounting for the CWD-relative skill resolution in unit test context.

    ### Suggested fix

    The test should either:
    1. Change CWD to the workspace root before calling `standardize_aibox_toml`, or
    2. Accept that in test context, skills have no catalog entry and adjust the assertion, or
    3. Mock/inject the skill catalog path in the test.

    ### Status

    851/852 tests pass. This failure should be tracked and fixed in a separate workitem before v0.25.7 ships.
  type: fleeting
  state: captured
  review_due: '2026-05-12'
  tags:
  - rust
  - test-failure
  - pre-existing
  - migration
  - skills
  - v0.25.7
  source: BACK-20260508_0629-GentleSeal-defer-rust-crate-updates
---

## Pre-existing test failure discovered during GentleSeal rust crate update pass

**Branch:** v0.25.7/gentleseal-rust-crate-updates
**Discovered:** 2026-05-10
**WorkItem:** BACK-20260508_0629-GentleSeal-defer-rust-crate-updates

### Failure

`migration::tests::standardize_aibox_toml_rewrites_schema_clean_config_to_canonical_shape` panics at `cli/src/migration.rs:2372`:

```
assertion failed: after.contains("    \"actor-profile\", # processkit;")
```

### Root cause

`skill_catalog_entries_for_comments()` resolves skill catalog entries from `current_dir().join("context/skills")`. When cargo runs unit tests, `current_dir()` is `cli/`, so it looks for `cli/context/skills/` which doesn't exist. The actor-profile skill entry is therefore not found in the catalog, causing `render_skill_array` to fall back to `"custom skill override"` instead of the expected `"processkit; ..."` comment format.

The test assertion `after.contains("    \"actor-profile\", # processkit;")` therefore fails.

### Not caused by

This failure is **not** caused by the Cargo.lock patch updates applied in this GentleSeal pass. The Cargo.lock diff is limited to transitive patch bumps (js-sys, wasm-bindgen family, cc, filetime, hashbrown). The `migration.rs` and `container.rs` files are untouched.

### Likely cause

A recent v0.25.7 merge (probably the skills catalog rendering feature or a test refactor) introduced this test without accounting for the CWD-relative skill resolution in unit test context.

### Suggested fix

The test should either:
1. Change CWD to the workspace root before calling `standardize_aibox_toml`, or
2. Accept that in test context, skills have no catalog entry and adjust the assertion, or
3. Mock/inject the skill catalog path in the test.

### Status

851/852 tests pass. This failure should be tracked and fixed in a separate workitem before v0.25.7 ships.
