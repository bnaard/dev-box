# aibox v0.32.2 — 2026-08-13

**Summary:** This patch completes the browser-testing host probe and substantially shortens future macOS release retries through safe, candidate-bound caching.

## Fixed

- Create an explicit Playwright `BrowserContext` before running axe-core, matching axe's supported integration contract.
- Preserve full-Chromium channel selection for the addon installed with Playwright `--no-shell`.

## Changed

- Reuse content-addressed container layers by default; `--cold-cache` remains available for deliberate clean rebuilds.
- Persist credential-free Cargo downloads and scope compiled macOS targets to the immutable candidate commit.
- Add `--retry-from=<failed-run-dir>` for checksummed reuse of successful conditional host probes from byte-identical candidate inputs, while lifecycle and security evidence remain fresh.

[v0.32.2]: https://github.com/projectious-work/aibox/compare/v0.32.1...v0.32.2
