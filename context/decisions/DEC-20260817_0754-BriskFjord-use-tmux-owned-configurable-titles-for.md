---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260817_0754-BriskFjord-use-tmux-owned-configurable-titles-for
  created: '2026-08-17T07:54:09+00:00'
spec:
  title: Use tmux-owned configurable titles for agent attention signaling
  state: accepted
  decision: Implement a provider-neutral aibox-agent-signal state contract backed
    by pane-scoped tmux options and window-level severity aggregation. Generate configurable
    Ghostty-visible tmux titles from aibox.toml, keep desktop notifications opt-in,
    and integrate harnesses through documented native hooks with explicit lifecycle
    or manual fallbacks where question detection is unavailable.
  context: The owner accepted the implementation plan for visual Ghostty tab notifications
    when an agent needs interaction inside an aibox tmux session.
  rationale: Tmux already sits between the harness and Ghostty and can safely own
    the terminal title. Explicit harness events are reliable; process-idle inference
    is not. A shared helper prevents provider-specific terminal logic and supports
    background panes.
  alternatives:
  - option: Infer attention from process idleness
    reason_rejected: Cannot distinguish thinking, waiting, rate limiting, or a user
      question reliably.
  - option: Run a macOS host daemon
    reason_rejected: Adds host dependencies and cross-container coordination when
      tmux and terminal protocols already provide the transport.
  - option: Let every harness emit OSC directly
    reason_rejected: Duplicates escape handling and cannot consistently aggregate
      background panes.
  consequences: aibox becomes the title owner inside tmux; competing shell title hooks
    must be disabled there. Harnesses without suitable hooks receive partial lifecycle
    or manual signaling rather than false automatic guarantees.
  related_workitems:
  - BACK-20260817_0753-GiftedCrow-tmux-agent-attention-titles
  decided_at: '2026-08-17T07:54:09+00:00'
---
