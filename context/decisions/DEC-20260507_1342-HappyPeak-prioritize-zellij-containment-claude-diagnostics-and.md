---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_1342-HappyPeak-prioritize-zellij-containment-claude-diagnostics-and
  created: '2026-05-07T13:42:55+00:00'
spec:
  title: Prioritize Zellij containment, Claude diagnostics, and tmux evaluation after
    runtime incident
  state: accepted
  decision: Proceed with the Zellij containment patch as the immediate stabilization
    path, defer the powerline/tabbar redesign to backlog, prioritize Claude Code runtime
    diagnostics next, and evaluate tmux as a possible aibox runtime multiplexer backend
    or default if Zellij remains unstable.
  context: On 2026-05-07 a derived aibox runtime showed OrbStack workload above 500%
    with evidence pointing at the Zellij server process. The sidecar WASM status plugin
    path also produced incorrect status data by reading the diagnostics sidecar cgroup
    instead of the main container. Claude Code diagnostics in the same derived project
    showed MCP auth/install drift and stale generated runtime state.
  rationale: 'The Zellij plugin system has too much blast radius for a default runtime
    component: a plugin or permission/render loop can pin the whole server. Stabilization
    must come before visual redesign. Claude Code drift is currently a user-facing
    setup failure and should be diagnosed/fixed before further UI polish. tmux deserves
    evaluation because its status line model is external-script based and easier to
    bound.'
  consequences: Sidecar status remains opt-in until stability gates exist. Powerline
    visual work remains backlog. The next implementation priority is a doctor/diagnose
    path for Claude Code and runtime drift, followed by an explicit tmux-vs-Zellij
    architecture recommendation.
  related_workitems:
  - BACK-20260507_1341-SoundSky-claude-code-derived-runtime-drift
  - BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign
  - BACK-20260507_1341-CalmEagle-evaluate-tmux-runtime-fallback
  - BACK-20260505_2222-KeenHare-investigate-zellij-status-plugin-errors
  decided_at: '2026-05-07T13:42:55+00:00'
---
