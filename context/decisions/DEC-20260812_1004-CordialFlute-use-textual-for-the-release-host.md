---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260812_1004-CordialFlute-use-textual-for-the-release-host
  created: '2026-08-12T10:04:50+00:00'
spec:
  title: Use Textual for the release host gate terminal UI
  state: accepted
  decision: Add Textual as an explicitly pinned host-gate dependency and use it for
    an interactive progress dashboard, while retaining a plain streaming renderer
    for non-TTY or explicitly disabled UI runs and preserving the existing append-only
    evidence logs as the source of truth.
  context: The owner accepted the previously discussed Textual dependency after repeated
    long-running macOS release rehearsals made it difficult to distinguish active
    work from hangs and to navigate mixed high-level status and verbose build output.
  rationale: Textual provides structured terminal layout, widgets, scrolling logs,
    keyboard interaction, and worker-safe message delivery without requiring tmux.
    A dual renderer keeps automation and redirected logs reliable, and the evidence
    files remain independent of terminal presentation.
  alternatives:
  - option: Continue ANSI heartbeat output
    reason_rejected: Does not provide persistent task overview, progress visualization,
      or navigable logs.
  - option: Use Rich only
    reason_rejected: Good transient rendering but would require custom application
      structure for focusable panes, scrolling, keybindings, and copy mode.
  - option: Use tmux panes
    reason_rejected: 'Adds nested session lifecycle and risks interfering with the
      developer terminal, contrary to issue #372.'
  consequences: The host bootstrap must pin and install Textual through uv before
    the credential-isolated gate starts. The UI must never be publication evidence
    itself, must degrade cleanly without a TTY, and must not suppress subprocess output
    from commands.log, command-results.log, or steps.log.
  related_workitems:
  - BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted
  decided_at: '2026-08-12T10:04:50+00:00'
---
