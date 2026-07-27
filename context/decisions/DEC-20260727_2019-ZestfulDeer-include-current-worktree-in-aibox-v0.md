---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260727_2019-ZestfulDeer-include-current-worktree-in-aibox-v0
  created: '2026-07-27T20:19:33+00:00'
spec:
  title: Include current worktree in aibox v0.28.16
  state: accepted
  decision: Release aibox v0.28.16 with the entire current worktree, including the
    processkit package-selection reconciliation and the Node.js installer repair.
  context: The owner explicitly authorized including all current worktree changes
    in the v0.28.16 patch release.
  rationale: Keeping the synchronized configuration, installed processkit surface,
    generated Dockerfile, addon source, and regression test together preserves a coherent
    reproducible release state.
  consequences: All current changes must be reviewed, committed, validated, and published
    together through the protected v0.x release workflow.
  decided_at: '2026-07-27T20:19:33+00:00'
---
