---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260425_0953-PluckyBadger-backlog-reprioritization-after-processkit
  created: '2026-04-25T09:53:41+00:00'
spec:
  title: Backlog reprioritization after processkit v0.19.1 upgrade (2026-04-25)
  state: accepted
  decision: "Establish 90-day focus on 5 items; defer remaining 37 to next planning cycle. Focus order: (1) Security review (JollyWren), (2) Pre-install three-way diff (DeepComet), (3) Version upgrade flows (LoyalSeal), (4) Preview companion design (CoolBear), (5) Process model retrospective (AmberWren) \u2014 parallel with above."
  context: "Audit of 42 backlog items post-v0.19.1 completed. HIGH priority items all confirmed Active. MEDIUM items split: 5 Active (unblocking), 18 Defer (90-day candidates), 4 Discuss (need owner input). LOW items: 9 Defer (6-12 month horizon), 2 Obsolete (remove). No circular dependencies found. Critical path: security \u2192 pre-install \u2192 version-upgrade-flows."
  rationale: Security review is foundational; must complete before finalizing any version-upgrade command. Pre-install three-way diff is a prerequisite for safe migrations. Version-upgrade flows design builds on both. Preview companion and process retrospective are high-value but don't block other work; can proceed in parallel. Deferring 37 items reduces cognitive load and allows focus on critical architectural work. Repository health is good (0 errors from pk-doctor); no urgent production fixes needed.
  related_workitems:
  - BACK-20260410_2236-JollyWren-security-review-cli-input
  - BACK-20260410_2236-DeepComet-pre-install-three-way
  - BACK-20260410_2236-LoyalSeal-version-upgrade-flows-review
  - BACK-20260410_2236-CoolBear-preview-companion-design-review
  - BACK-20260410_2236-AmberWren-process-model-retrospective-competitive
  decided_at: '2026-04-25T09:53:41+00:00'
---
