# aibox v0.32.3 — 2026-08-14

**Summary:** This patch makes the release-host browser fixture accessibility-clean, verified against the actual macOS Chromium host image.

## Fixed

- Add the missing level-one heading required by axe's page-level best-practice rules; the corrected fixture returns zero violations on the macOS release host.
- Preserve structured violation IDs, help URLs, affected HTML, and failure summaries in host evidence if the fixture regresses.

## Changed

- Keep the explicit Playwright `BrowserContext`, full Chromium channel, default cache reuse, and candidate-bound retry support introduced in v0.32.2.

[v0.32.3]: https://github.com/projectious-work/aibox/compare/v0.32.2...v0.32.3
