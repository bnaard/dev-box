---
apiVersion: processkit.projectious.work/v2
kind: Discussion
metadata:
  id: DISC-20260424_0008-CoolStream-which-of-the-42
  created: '2026-04-24T00:08:42+00:00'
spec:
  question: Which of the 42 backlog items remain relevant after processkit v0.19.1 upgrade, and what should our implementation priorities be?
  state: active
  opened_at: '2026-04-24T00:08:42+00:00'
  participants:
  - ACTOR-20260414_1100-CalmHawk-pm-agent
  - ACTOR-20260414_1100-NimbleMouse-junior-developer-agent
---

## Backlog Grooming Session

**Context:** The aibox project has 42 backlog items accumulated over several releases. Many reference outdated processkit versions (e.g., SoundRabbit references v0.8.0; we're now on v0.19.1). This session will audit and reprioritize the backlog.

### Key Items to Review

**CRITICAL Priority:**
- BACK-0000-SoundRabbit: "Adapt aibox + self-hosted devcontainer after processkit v0.8.0 overhaul" — References old version. Still needed after v0.19.1?

**HIGH Priority:**
- BACK-0000-AmberWren: Process model retrospective — strategic alignment
- BACK-0000-LoyalSeal: Version upgrade flows — system design  
- BACK-0000-CoolBear: Preview companion design
- BACK-0000-JollyWren: Security review — critical path

**MEDIUM Priority (sample):**
- BACK-20260417_1705-BoldAnchor: Soft reset of processes/skills
- BACK-0000-DeepComet: Pre-install three-way diff
- BACK-0000-SmartGlade: Skill eval framework

### Grooming Approach

**Pass 1: Audit (30 min)**
- Quick triage of each item: obsolete, defer, or active
- Flag items requiring context/discussion
- Note processkit version mismatches

**Pass 2: Dependencies (15 min)**
- Identify critical blockers
- Surface items waiting on upstream changes
- Clarify parent/child relationships

**Pass 3: Reprioritize (15 min)**
- Align with current project state
- Establish next 3-month focus area
- Decide what to tackle post-LuckyRabbit/EagerStone

### Success Criteria

- [ ] All items either validated as current or marked for review/removal
- [ ] Top 5 priority items clearly identified
- [ ] Blockers and dependencies documented
- [ ] Plan for next phase agreed
