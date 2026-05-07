---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1456-KeenComet-tmux-persistence-policy-resurrect-continuum
  created: '2026-05-07T14:56:07+00:00'
spec:
  title: Decide tmux persistence policy for resurrect and continuum
  state: backlog
  type: task
  priority: medium
  description: 'Determine whether tmux-resurrect and tmux-continuum are complementary,
    overlapping, or risky for aibox. Default implementation should install them pinned
    but keep persistence disabled unless a policy is accepted. Evaluate interaction
    with host-mounted tmux config, project-local state, AI panes, MCP proxy processes,
    and container recreation. Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  scope: runtime-architecture
---
