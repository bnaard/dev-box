---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260825_1100-MellowSky-adopt-opt-in-hugo-graphics-renderer
  created: '2026-08-25T11:00:30+00:00'
spec:
  title: Adopt opt-in Hugo graphics renderer addons
  state: accepted
  decision: 'Add three opt-in addons: diagramming for D2 and Graphviz, data-visualization
    for Vega and Vega-Lite CLI tooling, and mermaid for Mermaid CLI with its browser
    runtime. Keep Typst as the existing language addon and treat CeTZ as a Typst package
    rather than a separate binary addon. Do not make these mandatory dependencies
    of docs-hugo.'
  context: The Hugo graphics integration audit identified build-time renderers that
    are not yet represented as first-class aibox addons.
  rationale: Separating native diagram tools, Node-based visualization tools, and
    browser-heavy Mermaid rendering preserves opt-in weight, clear dependencies, reproducibility,
    and independent lifecycle management.
  alternatives:
  - option: One graphics-rendering addon
    reason: Rejected because addon-level dependencies would force Node and browser
      tooling on D2/Graphviz users.
  - option: Add all renderers to docs-hugo
    reason: Rejected because most Hugo projects do not need every renderer and the
      browser footprint is substantial.
  consequences: Each addon needs pinned versions, installation and purge behavior,
    architecture handling where applicable, smoke tests that render SVG, and documentation.
    Mermaid remains explicitly opt-in because of its browser footprint.
  decided_at: '2026-08-25T11:00:30+00:00'
---
