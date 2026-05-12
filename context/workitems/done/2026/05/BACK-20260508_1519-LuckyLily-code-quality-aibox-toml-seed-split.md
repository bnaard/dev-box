---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  created: '2026-05-08T15:19:10+00:00'
  labels:
    track: code-quality
    release: v0.25.6
  updated: '2026-05-08T21:28:30+00:00'
spec:
  title: "v0.25.6: Code quality \u2014 aibox.toml dedup and seed.rs split"
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-08T21:14:39+00:00'
  completed_at: '2026-05-08T21:28:30+00:00'
---

## Transition note (2026-05-08T21:14:39+00:00)

Q1 already shipped (commit ce35a4d). Now dispatching Q3 (seed.rs split into cli/src/tmux/) and Q4 (banner comments) to Avery. Q2/Q5/Q6/Q7 deferred to follow-up tracks.


## Transition note (2026-05-08T21:28:25+00:00)

Q1 (toml dedup) shipped in commit ce35a4d. Q3 (seed split) + Q4 (banners) shipped in commit (this batch). Q6 (slot order DEC) recorded as DEC-20260508_2115-SilentFern. Q2 (skills CLI), Q5 (CI workflow), Q7 (comment fact-check) deferred to follow-up tracks.


## Transition note (2026-05-08T21:28:30+00:00)

Accepted as done. Q2/Q5/Q7 follow-up tracks to file separately if needed.
