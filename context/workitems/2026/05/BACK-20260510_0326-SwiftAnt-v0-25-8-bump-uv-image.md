---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0326-SwiftAnt-v0-25-8-bump-uv-image
  created: '2026-05-10T03:26:09+00:00'
  labels:
    version: v0.25.8-candidate
    area: addons-python
spec:
  title: 'v0.25.8: bump uv image pin 0.11.11 → 0.11.12 (skip the intermediate to avoid
    two rebuilds)'
  state: backlog
  type: task
  priority: low
  description: |
    ## Background

    v0.25.7 shipped uv 0.11.10 → 0.11.11 (BACK-SureSeal, commit 0b31a2d, merge a9cbc00). PluckyEagle's drift review surfaced uv 0.11.12 as the next available patch and recommended in NOTE-20260509_2237-VastVale that the next pass go directly to 0.11.12 to amortize the base-image rebuild cost.

    ## Goal

    Bump uv 0.11.11 → 0.11.12 in:

    - `images/base-debian/Dockerfile` (COPY pin)
    - `addons/languages/python.yaml` (default_version, supported_versions)
    - `aibox.toml` (dogfood pin)
    - `cli/src/addon_registry.rs` (test assertions)
    - `cli/src/addon_cmd.rs` (test fixtures)
    - `cli/src/config.rs` (doc-comment example)
    - `scripts/release-check-state.sh` (uv_pin variable)
    - `docs-site/docs/addons/{overview,language-runtimes}.md`, `docs-site/docs/reference/configuration.md`

    ## Acceptance

    - 0.11.12 in all the above paths.
    - Base image rebuilds; aibox-level tests pass.

    ## Refs

    - NOTE-20260509_2237-VastVale (PluckyEagle's recommendation)
    - Commit 0b31a2d (SureSeal's 0.11.11 bump as the diff template)
---
