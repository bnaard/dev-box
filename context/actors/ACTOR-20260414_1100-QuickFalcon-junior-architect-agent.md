---
apiVersion: processkit.projectious.work/v2
kind: Actor
metadata:
  id: ACTOR-20260414_1100-QuickFalcon-junior-architect-agent
  created: 2026-04-14 11:00:00+00:00
spec:
  type: ai-agent
  name: Junior Architect Agent (template)
  active: true
  joined_at: 2026-04-14 11:00:00+00:00
  handle: junior-architect-agent
  is_template: true
  templated_from: null
  x_aibox:
    model: claude-sonnet-4-6
    model_tier: sonnet
    role_ref: ROLE-20260414_1100-QuickFalcon-junior-architect
---

Template actor for the junior-architect role. PM's default architect choice.
