# aibox v0.32.0 — 2026-08-13

**Summary:** This minor release adds an opt-in, reproducible browser-testing environment for projects that need responsive, keyboard, theme, reduced-motion, accessibility, and screenshot checks. Existing projects require no changes; enable the new addon when browser testing is wanted.

## Added

- Add the `browser-testing` addon with pinned Playwright Test 1.62.1 and `@axe-core/playwright` 4.13.0.
- Install Playwright-managed full Chromium by default, with Firefox and WebKit available as opt-in tools.
- Exercise a minimal Chromium launch and axe accessibility fixture in the macOS release-host gate.

## Changed

- Make the release-host progress bar span the viewport instead of matching only the task-list width.
- Document that derived projects own their application-specific responsive, keyboard-focus, light/dark, reduced-motion, accessibility, and visual-regression matrices.

## Fixed

- Remove the redundant outer border around the Textual log and the empty bordered legend row above Problems.
- Keep browser-testing package and browser installation consistent when individual addon tools are disabled.

[v0.32.0]: https://github.com/projectious-work/aibox/compare/v0.31.5...v0.32.0
