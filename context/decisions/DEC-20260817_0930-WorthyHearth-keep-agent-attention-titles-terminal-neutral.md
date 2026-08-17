---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260817_0930-WorthyHearth-keep-agent-attention-titles-terminal-neutral
  created: '2026-08-17T09:30:28+00:00'
spec:
  title: Keep agent-attention titles terminal-neutral
  state: accepted
  decision: 'Keep the agent-attention title contract terminal-emulator-neutral: tmux
    owns standard terminal titles, while optional attention notifications select an
    explicit osc-9 or bell protocol. Ghostty is one compatible terminal, not an architectural
    dependency.'
  context: The owner asked that the v0.x agent-attention title feature avoid binding
    aibox too closely to Ghostty.
  rationale: Standard tmux title propagation works across terminal emulators. Notification
    escape sequences vary, so representing the notification protocol explicitly and
    offering bell as a portable fallback separates the core title feature from terminal-specific
    capabilities.
  alternatives:
  - option: Keep a Ghostty-specific OSC 9 notification switch
    reason_rejected: It makes an optional terminal capability appear to be a core
      Ghostty dependency.
  - option: Remove notifications entirely
    reason_rejected: It would discard useful opt-in behavior even though the protocol
      boundary can be modeled explicitly.
  consequences: The unshipped notifications configuration uses enabled and protocol
    fields; documentation describes terminal capabilities rather than prescribing
    Ghostty. OSC 9 remains available for message-bearing notifications and bell is
    the portable fallback.
  related_workitems:
  - BACK-20260817_0753-GiftedCrow-tmux-agent-attention-titles
  decided_at: '2026-08-17T09:30:28+00:00'
---
