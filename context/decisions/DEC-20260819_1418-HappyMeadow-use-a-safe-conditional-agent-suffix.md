---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260819_1418-HappyMeadow-use-a-safe-conditional-agent-suffix
  created: '2026-08-19T14:18:11+00:00'
spec:
  title: Use a safe conditional agent suffix placeholder in tmux titles
  state: accepted
  decision: Add {agent_suffix} as a first-class title placeholder. It renders ' —
    model@harness' when both values exist, ' — harness' when only the harness exists,
    and an empty string when no harness exists.
  context: Literal separators around {agent} and {harness} remain visible when those
    values are empty on non-agent windows, while raw tmux format expressions are intentionally
    escaped for safety.
  rationale: This provides clean optional rendering without exposing arbitrary tmux
    format syntax or weakening title literal escaping.
  consequences: Configuration validation, generated tmux expressions, option catalogs,
    tests, and documentation must recognize the new placeholder.
  related_workitems:
  - BACK-20260819_1417-MerryPath-conditional-tmux-agent-title-suffix
  decided_at: '2026-08-19T14:18:11+00:00'
---
