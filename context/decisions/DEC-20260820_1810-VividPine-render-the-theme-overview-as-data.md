---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260820_1810-VividPine-render-the-theme-overview-as-data
  created: '2026-08-20T18:10:35+00:00'
spec:
  title: Render the theme overview as data-driven Hugo HTML
  state: accepted
  decision: Publish one discoverable Hugo theme overview generated from the same audited
    theme data used by aibox runtime scaffolding. Render semantic characteristics,
    tool support, and terminal specimens as HTML/CSS rather than screenshots or asciinema
    recordings.
  context: The documentation needs a complete, maintainable overview of every shipped
    theme. Screenshot and terminal-recording pipelines add unnecessary complexity
    and are not required to express exact palette values.
  rationale: A single generated data file and Hugo template keep documentation aligned
    with effective theme output, remain responsive and accessible, and avoid host-terminal
    and font-dependent raster rendering.
  consequences: The existing hidden screenshot gallery becomes obsolete. Release validation
    should eventually check generated theme data for drift; runtime tmux visual tests
    remain separate from documentation rendering.
  decided_at: '2026-08-20T18:10:35+00:00'
---
