---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260819_1234-CordialClover-populate-tmux-agent-titles-from-harness
  created: '2026-08-19T12:34:09+00:00'
spec:
  title: Populate tmux agent titles from harness-specific active model state
  state: accepted
  decision: Implement harness-specific active-model adapters for all supported aibox
    harnesses. Use native model/session hooks where available, and only append effort
    or mode when the harness exposes it reliably. Avoid presenting defaults as active
    session state when models can change mid-session.
  context: The owner accepted the audit recommendation to extend the existing Codex
    and Claude title behavior to the remaining supported harnesses.
  rationale: Harnesses expose model identity through different lifecycle, plugin,
    transcript, configuration, and session mechanisms. Per-harness adapters preserve
    correctness while keeping the title format provider-neutral.
  consequences: Gemini and OpenCode can use native runtime integrations first. Copilot,
    Cursor, Aider, Continue, Hermes, and Tau require guarded adapters appropriate
    to their available state. Full style degrades to model-only when no reliable effort
    value exists.
  decided_at: '2026-08-19T12:34:09+00:00'
---
