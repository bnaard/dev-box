---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260818_1919-FocusedLeaf-prioritize-native-attention-adapters-without-tau
  created: '2026-08-18T19:19:40+00:00'
spec:
  title: Prioritize native attention adapters without Tau upstream work
  state: accepted
  decision: Implement and validate native attention lifecycle adapters for OpenCode,
    Gemini CLI, and GitHub Copilot CLI. Do not pursue Tau integration or upstream
    interaction in this scope.
  context: The owner confirmed Claude and Codex behavior and selected the next supported
    harnesses. Tau lacks a documented external lifecycle interface in the pinned release
    and was explicitly excluded.
  rationale: These three harnesses expose documented native lifecycle events suitable
    for deterministic working, question, done, error, and idle status transitions
    without terminal scraping.
  consequences: aibox will generate and merge provider-specific lifecycle configuration
    for OpenCode, Gemini CLI, and Copilot CLI. Tau remains launcher-idle only.
  related_workitems:
  - BACK-20260818_1919-AgileBird-native-attention-opencode-gemini-copilot
  decided_at: '2026-08-18T19:19:40+00:00'
---
