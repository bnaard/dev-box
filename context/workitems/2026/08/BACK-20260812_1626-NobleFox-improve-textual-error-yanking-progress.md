---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260812_1626-NobleFox-improve-textual-error-yanking-progress
  created: '2026-08-12T16:26:28+00:00'
spec:
  title: Improve release-host Textual selection, error yanking, and progress clarity
  state: backlog
  type: story
  priority: medium
  description: |
    Improve the release-host Textual dashboard based on the v0.31.3 host run.

    Acceptance criteria:
    - The log widget supports ordinary mouse drag selection as well as keyboard selection.
    - Keep Select All, and add a documented hotkey that selects approximately the last 20 visible/logical log lines (use a named constant so the count is testable and adjustable).
    - The normal yank/copy action copies only the current marked selection. It must not silently copy the complete task log when there is no selection; instead show a concise no-selection notification.
    - Add a dedicated Yank Errors hotkey. It copies a concise chronological error bundle containing failed task states, warning task states where relevant, and captured diagnostic/error lines, with enough task context to report the failure. It must not copy unrelated successful build output.
    - Define and test error-line classification so ANSI/control sequences, candidate markup, ordinary progress output, and warning-only policy results cannot cause misleading copied text.
    - Make overall progress determinate from application startup: show completed/total task count and percentage immediately. Conditional tasks may begin as pending and later become skipped; dynamic cleanup tasks must not make the denominator move backward or produce a misleading percentage.
    - Replace unexplained task glyphs with a documented legend and accessible text/state. Distinguish pending circle, running spinner, passed, warning/exclamation, skipped, and failed.
    - Add a compact Problems summary surface listing warnings and failures. Selecting a problem navigates or filters to the relevant task log. Yank Errors uses this same canonical problem model rather than a separate parser.
    - Warning states such as accepted no-fix vulnerabilities remain visibly distinct from failures and explain why publication may continue.
    - Headless tests cover mouse/selection behavior where Textual supports it, Select All, last-20 selection, selection-only yank, empty-selection handling, error-only yank, initial numeric progress, conditional skips, and warning/failure summaries at narrow and wide terminal sizes.
    - Plain UI behavior and authoritative evidence files remain unchanged; clipboard/UI state is never release evidence.

    Related implementation: scripts/release_host_gate.py and scripts/test-release-host-gate.sh. Related decision/spec: DEC-20260812_1004-CordialFlute and NOTE-20260812_1013-AmpleTulip.
---
