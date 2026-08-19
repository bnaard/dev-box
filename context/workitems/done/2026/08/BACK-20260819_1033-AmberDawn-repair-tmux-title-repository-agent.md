---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1033-AmberDawn-repair-tmux-title-repository-agent
  created: '2026-08-19T10:33:26+00:00'
  updated: '2026-08-19T10:49:00+00:00'
spec:
  title: Repair tmux title repository and agent identity rendering
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Add configurable repository basename versus owner/repository title
    rendering across standard Git remote URL forms, retain repository metadata while
    idle, auto-resolve agent identity, and cover generated/runtime/docs surfaces.
  started_at: '2026-08-19T10:33:32+00:00'
  completed_at: '2026-08-19T10:49:00+00:00'
---

## Transition note (2026-08-19T10:33:32+00:00)

Implementation started after reproducing empty agent, workspace basename, and idle metadata loss.


## Transition note (2026-08-19T10:49:00+00:00)

Implementation complete. Focused title tests, cross-forge remote cases, full Rust suite product checks, clippy, shell syntax, Hugo build, live apply, and live tmux basename/full rendering verified. One unrelated visual Yazi test times out only under the full parallel E2E suite and passes in isolation.


## Transition note (2026-08-19T10:49:00+00:00)

Accepted by verification evidence; live title now renders aibox — Avery@codex and full mode renders projectious-work/aibox — Avery@codex.
