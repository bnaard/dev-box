# aibox v0.34.4 — 2026-08-21

**Summary:** This follow-up patch updates the bundled Yazi pane-toggle plugin to the current pane-ratio API, eliminating deprecation warnings found during the v0.34.3 visual release gate.

## Fixed

- Read Yazi's parent, current, and preview ratios through the supported indexed API.
- Preserve pane hide, maximize, restore, and reset behavior without deprecated field access.
- Add a regression test that rejects the deprecated ratio field names in the generated plugin.
- Register v0.34.4 against processkit v0.28.8 in the CLI and documentation compatibility tables.

## Upgrade notes

Upgrade the host CLI to v0.34.4, run `aibox apply`, and restart existing Yazi processes so they load the refreshed plugin.

[v0.34.4]: https://github.com/projectious-work/aibox/compare/v0.34.3...v0.34.4
