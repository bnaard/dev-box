---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_0511-EagerDew-docs-addons-project-local-npm-install
  created: '2026-05-09T05:11:22+00:00'
  labels:
    track: addons
    release: v0.25.7
    surfaced_during: v0.25.6 release Phase 1 docs-deploy
  updated: '2026-05-09T22:19:07+00:00'
spec:
  title: 'v0.25.7: docs addons should run project-local `npm install` (prism-react-renderer
    surprise)'
  state: done
  type: task
  priority: medium
  description: |
    ## Trigger

    During v0.25.6 release Phase 1 docs-deploy (2026-05-09), `cmd_docs_deploy` failed with:

    ```
    Error: Docusaurus could not load module at path "/workspace/docs-site/docusaurus.config.js"
    Cause: Cannot find module 'prism-react-renderer'
    ```

    `docs-site/package.json` correctly lists `prism-react-renderer ^2.3.0` as a direct dep. `docs-site/node_modules/` was empty. Manual `cd docs-site && npm install` resolved it; the release docs deployment succeeded on retry.

    ## Architecture gap

    `addons/docs/docs-docusaurus.yaml` runtime block installs Docusaurus **globally** in the container image:

    ```yaml
    runtime: |
      # Addon: docs-docusaurus
      RUN npm install -g @docusaurus/core@{{ tools.docusaurus.version }} && \
          npm cache clean --force
    ```

    This is right for the global CLI but does NOT install project-local deps (`prism-react-renderer`, `react`, `@docusaurus/preset-classic`, etc. listed in `docs-site/package.json`). Other docs addons (Hugo, mdBook, MkDocs, Starlight, Zensical) have the same shape.

    The closest existing hook is `post_create_command` in `cli/src/config.rs` (commented as an example in the generated `aibox.toml`), but that's per-project opt-in, not addon-driven, and was not set in this project's `aibox.toml`.

    ## Three fix paths (owner picks one)

    ### Option A — Project-level fix (cheapest, single project)
    Add `post_create_command = "cd docs-site && npm install"` (or equivalent) to *this* project's `aibox.toml`. Doesn't help any other project using the same addon.

    ### Option B — Addon-level fix (correct in principle)
    Extend `docs-docusaurus.yaml` (and other docs addons) with a new `project_post_apply` hook in the addon schema. Hook runs after container is up. Convention: addon declares its docs-directory pattern (e.g., `docs-site/`, `docs/`, `site/`) and aibox runs `npm install` (or `pip install`, etc.) if a corresponding manifest is detected.

    Touches `cli/src/addon_loader.rs` (schema + execution) plus every docs addon yaml.

    ### Option C — Release-script-level safety net (cheapest correctness fix)
    `cmd_docs_deploy` in `scripts/maintain.sh` runs `npm install` (or `npm ci`) first if `node_modules` is missing or stale relative to `package.json`/`package-lock.json`. Single-place fix, doesn't generalize beyond docs deploy.

    ## Recommendation

    **Option C as the immediate safety net** (15 min), AND **Option B as the architectural fix** (medium scope, addon-system change). Option A is a workaround, not a fix.

    ## Side findings to track separately if non-trivial
    - 25 npm-audit vulnerabilities reported (3 moderate, 22 high) when running `npm install` in `docs-site/`. Probably downstream of Docusaurus stack; investigate severity before triage.
    - Docusaurus 3.9.2 → 3.10.1 upgrade available (warning printed during build).
    - Pre-existing clippy issues (`src/lock.rs:965` field-reassign-with-default, `tests/e2e/addon_disablement.rs:546` useless_format) flagged by Avery — not blocking the v0.25.6 release because `cmd_test` runs `cargo clippy --` (not `--all-targets`), but worth a single-pass cleanup.

    ## Dispatch hint

    - Option C: Robin (junior eng / mechanical), 15-30 min.
    - Option B: Sage (CTO) for the schema decision, then Avery for implementation, ~1-2h.

    ## Why this is v0.25.7 and not v0.25.6

    v0.25.6 has shipped (tag pushed, GitHub release created, docs deployed via the manual `npm install` workaround). The architectural fix doesn't unblock anything that's broken today; it prevents a future agent or human from hitting the same surprise.
  started_at: '2026-05-09T22:18:28+00:00'
  completed_at: '2026-05-09T22:19:07+00:00'
---

## Transition note (2026-05-09T22:19:07+00:00)

Implemented and merged in commit fac2aa1 + merge b36241f. New cli/src/docs_install.rs (~300 LOC + 11 unit tests), wired in container.rs::cmd_sync.
