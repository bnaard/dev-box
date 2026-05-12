---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260424_0042-QuietOak-escalate-processkit-v0-19
  created: '2026-04-24T00:42:41+00:00'
spec:
  title: Escalate processkit v0.19.1 collision issues to upstream
  state: accepted
  decision: 'Report two collision issues discovered during derived project aibox sync with processkit v0.19.1: (1) duplicate skill basename ''retrospective'' (product vs processkit), (2) duplicate command ''pk-resume.md'' (morning-briefing vs status-briefing). Created GitHub issues with processkit project requesting upstream fix and documenting workarounds for affected users.'
  context: 'Derived project reported during `aibox sync`: warnings about skill and command collisions resolved via last-wins semantics. Silent functionality loss for users if they expect the product-team''s retrospective or morning-briefing''s pk-resume.'
  rationale: 'These are upstream processkit issues that affect all derived projects on v0.19.1. Creating issues with upstream team ensures: (1) visibility of the problem, (2) coordinated fix across skill owners, (3) prevention of similar collisions in future releases via CI tests, (4) clear guidance for affected users on workarounds/pinning.'
  decided_at: '2026-04-24T00:42:41+00:00'
---
