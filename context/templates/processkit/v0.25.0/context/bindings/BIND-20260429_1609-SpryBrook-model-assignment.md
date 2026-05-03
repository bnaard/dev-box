---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1609-SpryBrook-model-assignment
  created: '2026-04-29T16:09:45+00:00'
spec:
  type: model-assignment
  subject: TEAMMEMBER-cora
  target: MODEL-openai-gpt-5
  conditions:
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior PM planning and acceptance using OpenAI equivalent to Claude
      Sonnet for Codex/OpenAI-subscription sessions.
  description: OpenAI preference for Cora in index-search implementation planning
---
