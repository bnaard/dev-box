# aibox v0.32.1 — 2026-08-13

**Summary:** This patch completes the browser-testing release by aligning the macOS host probe with the full Chromium binary installed by the addon. Projects using the addon need no configuration change.

## Fixed

- Launch Playwright with `channel: "chromium"` in the release-host fixture so `--no-shell` installations use full Chromium rather than searching for the intentionally omitted headless-shell executable.
- Add a contract assertion that prevents the host probe from drifting back to the default headless-shell launch mode.

[v0.32.1]: https://github.com/projectious-work/aibox/compare/v0.32.0...v0.32.1
