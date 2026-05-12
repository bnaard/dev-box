---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_2216-StoutDew-use-list-based-tmux-powerkit-status
  created: '2026-05-11T22:16:25+00:00'
spec:
  title: Use list-based tmux PowerKit status layout configuration
  state: accepted
  decision: Add a list-based tmux status layout model where status rows are configured by ordered plugin-name lists, while preserving the existing boolean element switches as backward-compatible input.
  context: The current tmux PowerKit status configuration exposes many booleans in aibox.toml while renderer placement is controlled by hard-coded row arrays. This can drift, as seen with exposed switches not affecting rendered plugins.
  rationale: Ordered lists make both enablement and placement explicit in aibox.toml, reduce drift between public switches and renderer slots, and give users direct control over row composition without adding new booleans for every placement change.
  alternatives:
  - option: Keep boolean switches only
    tradeoff: Smallest schema but placement remains hidden and drift-prone.
  - option: Replace booleans immediately
    tradeoff: Cleaner schema but breaks existing derived projects.
  consequences: The implementation should accept the new ordered lists when present, continue to honor legacy booleans for existing configs, validate unknown plugin names, and standardize configs toward the list-based layout.
  decided_at: '2026-05-11T22:16:25+00:00'
---
