---
apiVersion: processkit.projectious.work/v2
kind: Actor
metadata:
  id: ACTOR-20260414_1100-SteadyOtter-developer-agent
  created: 2026-04-14 11:00:00+00:00
spec:
  type: ai-agent
  name: Developer Agent (template)
  active: true
  joined_at: 2026-04-14 11:00:00+00:00
  handle: developer-agent
  is_template: true
  templated_from: null
  x_aibox:
    model: claude-sonnet-4-6
    model_tier: sonnet
    role_ref: ROLE-developer
---

Template actor for the developer role. Main execution role; PM spawns clones for independent parallel subtasks.
