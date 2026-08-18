---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260818_1919-AgileBird-native-attention-opencode-gemini-copilot
  created: '2026-08-18T19:19:32+00:00'
  updated: '2026-08-18T19:30:58+00:00'
spec:
  title: Add native attention hooks for OpenCode Gemini and Copilot
  state: done
  type: story
  priority: high
  description: Complete and validate native tmux attention lifecycle mappings for
    OpenCode, Gemini CLI, and GitHub Copilot CLI. Preserve user-owned hook configuration
    and add regression coverage. Tau is explicitly out of scope.
  started_at: '2026-08-18T19:19:46+00:00'
  completed_at: '2026-08-18T19:30:58+00:00'
---

## Transition note (2026-08-18T19:19:46+00:00)

Implementation started for OpenCode, Gemini CLI, and Copilot CLI native attention adapters; Tau excluded by owner decision DEC-20260818_1919-FocusedLeaf.


## Transition note (2026-08-18T19:30:53+00:00)

Implemented native attention adapters for OpenCode, Gemini CLI, and GitHub Copilot CLI; Tau excluded by decision. Focused tests, full clippy/fmt, pk-doctor, and serialized reruns of three contention-prone visual tests pass.


## Transition note (2026-08-18T19:30:58+00:00)

Accepted scope is complete: OpenCode explicit question events, Gemini lifecycle hooks, Copilot CLI hooks, documentation, and regression coverage. No Tau interaction or changes.
