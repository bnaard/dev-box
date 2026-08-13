---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260813_1417-FastTrail-provide-a-chromium-first-playwright-and
  created: '2026-08-13T14:17:27+00:00'
spec:
  title: Provide a Chromium-first Playwright and axe browser-testing addon
  state: accepted
  decision: Add a v0.x `browser-testing` tool addon built around a pinned, coherent
    Playwright Test and `@axe-core/playwright` installation with Playwright-managed
    full Chromium enabled by default. Firefox and WebKit are optional tools. The aibox
    repository will validate installation, rendering, browser launch, configuration
    toggles, and a minimal fixture; derived projects own their application-specific
    viewport, color-scheme, motion, keyboard, accessibility, and screenshot matrix.
  context: Derived projects need reproducible headless testing for responsive layouts,
    keyboard focus, light and dark color schemes, reduced motion, visual regression
    screenshots, and automated accessibility checks. The addon must minimize dependencies
    while keeping upstream-supported integration and reproducible browser/tool version
    coupling. aibox itself owns the addon contract, not each derived application's
    visual coverage matrix.
  rationale: The Node Playwright stack is the smallest maintained stack that combines
    a runner, browser isolation, emulation, keyboard control, traces, screenshot baselines,
    and Deque's maintained axe adapter. Python or Selenium alternatives avoid npm
    only superficially while requiring more independent packages and weaker visual-regression
    integration. Keeping aibox tests to a minimal contract avoids turning the CLI
    repository into a sample web application's screenshot suite.
  alternatives:
  - option: Python Playwright plus pytest and separate image/axe integrations
    reason_rejected: More independent dependencies and no equally integrated official
      golden-screenshot plus axe stack.
  - option: Selenium or raw Chromium CLI
    reason_rejected: Requires more driver/assertion/diff infrastructure and provides
      weaker integrated emulation, traces, and visual baselines.
  - option: Run a full responsive/theme/motion screenshot matrix in aibox
    reason_rejected: That matrix validates a derived application, while aibox should
      validate only the addon installation and configuration contract.
  consequences: The addon introduces an accepted npm/Node dependency and a sizable
    Chromium browser layer. Browser revisions must remain coupled to the pinned Playwright
    version. aibox must provide focused addon rendering and launch smoke tests, plus
    optional-browser enable/disable coverage. Documentation should show a representative
    derived-project matrix without requiring that matrix in aibox CI.
  deciders:
  - TEAMMEMBER-thrifty-otter
  decided_at: '2026-08-13T14:17:27+00:00'
---
