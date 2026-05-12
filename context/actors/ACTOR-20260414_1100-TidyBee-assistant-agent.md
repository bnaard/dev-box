---
apiVersion: processkit.projectious.work/v2
kind: Actor
metadata:
  id: ACTOR-20260414_1100-TidyBee-assistant-agent
  created: 2026-04-14 11:00:00+00:00
spec:
  type: ai-agent
  name: Assistant Agent (template)
  active: true
  joined_at: 2026-04-14 11:00:00+00:00
  handle: assistant-agent
  is_template: true
  templated_from: null
  x_aibox:
    model: claude-haiku-4-5-20251001
    model_tier: haiku
    role_ref: ROLE-20260414_1100-TidyBee-assistant
---

Template actor for the assistant role. High-volume administrative work: morning briefings, summaries, indexing, handovers.
