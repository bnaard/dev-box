---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_0714-StoutFern-selective-visual-e2e-release-gating-and
  created: '2026-05-06T07:14:03+00:00'
spec:
  title: Selective visual e2e release gating and screenshot reuse
  state: accepted
  decision: 'aibox release validation will use selective e2e visual gates: agents must judge whether a release changed runtime, layout, theme, terminal-plugin, tool, harness, or image surfaces and run the relevant visual e2e tier when it did; the full visual e2e matrix remains mandatory at least every fifth release; visual e2e fixtures should also support producing current-release documentation screenshots.'
  context: The full SSH/asciinema e2e companion suite is intentionally realistic and therefore expensive. The project still needs strong regression coverage for Zellij native status rows, generated layouts, Yazi previews, themes, and enabled tools, but not every release changes those surfaces. The same captures can reduce documentation drift if they can be exported as website screenshots.
  rationale: This keeps release validation evidence-based without making every patch release pay the full visual-matrix cost. Periodic full sweeps catch drift across unchanged-looking surfaces, while screenshot reuse ensures docs reflect the current generated runtime rather than hand-maintained images.
  alternatives:
  - option: Run full visual e2e on every release
    tradeoff: Highest confidence but slow and likely to discourage frequent patch releases.
  - option: Only run focused tests when a bug is suspected
    tradeoff: Fastest but misses silent regressions in generated layouts, themes, and docs visuals.
  consequences: Release instructions need a judgment checkpoint and a periodic full-sweep rule. E2E visual tests need separable status/theme, tool/tab traversal, Yazi preview, and documentation-capture modes. Release notes or handover should state which e2e tier was run or why it was skipped.
  deciders:
  - ACTOR-20260410_2209-SnappyFrog-bernhard
  decided_at: '2026-05-06T07:14:03+00:00'
---
