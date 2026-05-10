---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260429_1630-SmartBison-ship-yazi-data-navigation
  created: '2026-04-29T16:30:19+00:00'
spec:
  title: Ship Yazi Data Navigation Addon
  state: accepted
  decision: 'Implement Yazi support as a repo-owned data/navigation feature set: native
    yank keybindings, a vendored/local what-size plugin, and a data-preview addon
    for SQLite plus CSV/TSV/XLS/XLSX previews. Exclude CLV handling because it was
    a typo.'
  context: The user accepted the implementation plan for additional Yazi features
    in aibox. The feature spans generated Yazi config, container addon dependencies,
    and seeded plugins, so it is a cross-cutting product/configuration decision.
  rationale: 'This keeps aibox provider-neutral and reproducible: Yazi features are
    generated from aibox configuration and shipped container files rather than relying
    on user-local harness state. CSV/TSV, SQLite, and spreadsheet previews belong
    behind an explicit addon because they add runtime tools/dependencies.'
  alternatives:
  - option: Document manual user setup only
    reason_not_chosen: Would not be reproducible across derived projects.
  - option: Always install all data preview dependencies in the base image
    reason_not_chosen: Increases base image footprint for users who do not need database/spreadsheet
      previewing.
  - option: Use only upstream Yazi plugins
    reason_not_chosen: Vendored/local small plugins match the existing aibox seed
      pattern and reduce moving parts for generated environments.
  consequences: Derived projects can opt into richer Yazi data previews through an
    addon while all projects get lightweight clipboard/yank shortcuts. Future preview
    formats should be added to the same addon unless they are cheap and broadly useful.
  decided_at: '2026-04-29T16:30:19+00:00'
---
