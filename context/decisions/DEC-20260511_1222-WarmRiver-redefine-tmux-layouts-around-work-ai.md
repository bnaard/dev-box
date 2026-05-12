---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1222-WarmRiver-redefine-tmux-layouts-around-work-ai
  created: '2026-05-11T12:22:01+00:00'
spec:
  title: Redefine tmux layouts around work ai lazygit shell windows
  state: accepted
  decision: 'aibox will replace the current tmux layout set with four layouts only: ai, dev, focus, and cowork. Layout placement uses harness_order semantics for the 1st and subsequent harnesses. The removed layouts are browse and cowork-swap.'
  context: The previous tmux-era layouts carried migration-era behavior and hidden secondary panes. The user provided a fresh layout specification with explicit window names and placement rules for work, ai, lazygit, and shell surfaces.
  rationale: A smaller layout set with explicit named windows is easier to reason about and creates stable semantics for upcoming layout work. Keeping all secondary harnesses visible in dedicated panes/windows avoids hidden-pane behavior.
  consequences: Config validation, generated layout scripts, docs, and tests must be updated to only expose ai/dev/focus/cowork. Existing configs using removed layouts need migration or fallback behavior.
  decided_at: '2026-05-11T12:22:01+00:00'
---
