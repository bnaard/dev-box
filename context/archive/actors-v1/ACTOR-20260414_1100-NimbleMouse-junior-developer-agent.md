---
apiVersion: processkit.projectious.work/v1
kind: Actor
metadata:
  id: ACTOR-20260414_1100-NimbleMouse-junior-developer-agent
  created: 2026-04-14T11:00:00Z
spec:
  type: agent
  name: Junior Developer Agent (template)
  active: true
  joined_at: 2026-04-14T11:00:00Z
  handle: junior-developer-agent
  is_template: true
  templated_from: null
  x_aibox:
    model: claude-haiku-4-5-20251001
    model_tier: haiku
    role_ref: ROLE-20260414_1100-NimbleMouse-junior-developer
---

Template actor for the junior-developer role. Used for well-specified mechanical changes; common fan-out target for bulk patterns.
