---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260819_1658-AgileCedar-use-brand-theme-hugo-vanilla-v0
  created: '2026-08-19T16:58:41+00:00'
spec:
  title: Use brand-theme-hugo-vanilla v0.3.4 as the aibox documentation foundation
  state: accepted
  decision: Replace Docsy/Hextra and their dependencies entirely with brand-theme-hugo-vanilla
    v0.3.4. Use the theme's Hugo-native configuration, layouts, partials, shortcodes,
    data, and content conventions wherever available. Preserve current documentation
    content with minimal editorial changes, rename Blog to Change log, add a visually
    related Roadmap beginning with v0.x and v1.x phases, and track any unavoidable
    bare HTML/CSS or hard-coded visual workarounds for an upstream issue.
  context: The existing GitHub Pages documentation uses Hugo Docsy/Hextra and their
    dependency stack. The owner chose the projectious.work brand implementation as
    the new authoritative design and Hugo foundation before the next release.
  rationale: This aligns aibox documentation with the authoritative projectious.work
    brand, reduces bespoke styling and legacy theme dependency, and feeds missing
    reusable constructs back into the shared theme rather than accumulating local
    forks.
  alternatives:
  - option: Continue maintaining Docsy/Hextra with local brand overrides
    rejected_because: Retains obsolete dependencies and duplicates the shared brand
      implementation.
  - option: Copy the reference site's rendered HTML/CSS into aibox
    rejected_because: Would bypass Hugo-idiomatic theme constructs, create drift,
      and prevent reusable upstream improvements.
  consequences: The docs information architecture and templates will change while
    content is largely preserved. Existing Docsy/Hextra modules, assets, and build
    dependencies must be removed. Local exceptions must be explicitly documented and
    reported upstream. Local preview must bind port 1316 for host verification.
  deciders:
  - ACTOR-owner
  - TEAMMEMBER-avery
  related_workitems:
  - BACK-20260819_1648-StableJay-establish-new-documentation-basis
  decided_at: '2026-08-19T16:58:41+00:00'
---
