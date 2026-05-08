---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_0425-BoldTiger-adopt-amended-two-line-tmux-status
  created: '2026-05-08T04:25:44+00:00'
  updated: '2026-05-08T04:25:52+00:00'
spec:
  title: Adopt amended two-line tmux status bar layout
  state: accepted
  decision: Use a two-line tmux-powerkit status bar for the tmux runtime. Line 1 is
    identity, navigation, and project/runtime context. Line 2 is mode, runtime health,
    and agent/processkit state. Use short word labels such as CPU, MEM, DISK, NET,
    OOM, PROC, AI, MCP, LOG, and MIG rather than ambiguous symbols. DISK must render
    used/total, not free-only.
  rationale: The current visible status field is the hostname plugin (house icon plus
    aibox), which is misleading as a runtime-status indicator. A two-line structure
    gives room for both navigation and operational health while keeping each row scannable.
    Short word labels are clearer than font-dependent symbols under load or during
    incident triage.
  consequences: The hostname presentation should be replaced or demoted to an explicit
    HOST field. The diagnostics sidecar/aibox-status collector needs additional fields
    for disk used/total, log severity counts, CPU usage deltas, and compact status
    rendering. Heavy optional powerkit plugins should remain opt-in or cheap-by-default.
  related_workitems:
  - BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  - BACK-20260508_0356-QuickGarnet-implement-standard-tmux-powerkit-keybindings-popup
  - BACK-20260508_0425-SilentCrane-implement-amended-two-line-tmux-status
  decided_at: '2026-05-08T04:25:44+00:00'
---
