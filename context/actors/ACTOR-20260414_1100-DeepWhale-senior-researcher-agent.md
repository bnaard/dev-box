---
apiVersion: processkit.projectious.work/v2
kind: Actor
metadata:
  id: ACTOR-20260414_1100-DeepWhale-senior-researcher-agent
  created: 2026-04-14 11:00:00+00:00
spec:
  type: ai-agent
  name: Senior Researcher Agent (template)
  active: true
  joined_at: 2026-04-14 11:00:00+00:00
  handle: senior-researcher-agent
  is_template: true
  templated_from: null
  x_aibox:
    model: claude-opus-4-6
    model_tier: opus
    role_ref: ROLE-senior-researcher
---

Template actor for the senior-researcher role. Invoked by PM for research requiring Opus-tier synthesis.
