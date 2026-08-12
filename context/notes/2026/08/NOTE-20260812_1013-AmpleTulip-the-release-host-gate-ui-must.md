---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260812_1013-AmpleTulip-the-release-host-gate-ui-must
  created: '2026-08-12T10:13:27+00:00'
spec:
  title: The release host gate UI must separate Textual presentation from authoritative
    evidence
  body: |
    # Textual release-host gate UI specification

    ## Status and relationships

    - Status: accepted for implementation
    - Decision: `DEC-20260812_1004-CordialFlute-use-textual-for-the-release-host`
    - WorkItem: `BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted`
    - GitHub: issue #372, PR #373
    - Target entry point: `./scripts/maintain.sh release-host <run-dir> [--dry-run]`

    ## Objective

    Make multi-minute macOS release-host validation visibly active, navigable, and copyable without changing its execution authority, evidence contract, or publication boundary.

    The interactive terminal must provide:

    1. a truthful overall progress bar;
    2. a persistent high-level task list with state and elapsed time;
    3. a bordered, scrollable live-output view whose text can be selected and copied between terminal windows.

    ## Source-layout constraint

    Keep all production Python implementation in the existing `scripts/release_host_gate.py`.

    That file owns:

    - gate execution;
    - the event and state model;
    - the plain renderer;
    - the Textual application;
    - inline Textual CSS;
    - task definitions and status reduction.

    Do not add a second production Python module. Existing shell entry points and shell contract tests may change. A dependency lock file is permitted and does not violate the one-Python-file constraint. The publisher remains in its existing separate `scripts/release_host_publish.py` and must not import Textual.

    ## Dependency contract

    - Pin Textual to an exact reviewed version and resolve all transitive packages in a committed lock artifact.
    - The trusted `release-host-gate.sh` wrapper provisions that locked environment through the already approved uv executable and exact Python 3.14.6.
    - Do not enable candidate-controlled PEP 723 metadata.
    - Record Python, uv, Textual, and resolved dependency provenance in `evidence/darwin-build/toolchain.json`.
    - Publication remains offline and Textual-free.

    ## Presentation selection

    Support `--ui=auto|textual|plain` at the maintain and host-gate entry points.

    - `auto` is the default.
    - `auto` uses Textual only when stdin and stdout are suitable TTYs and `TERM` is not `dumb`.
    - `auto` falls back to plain rendering for pipes, redirected output, automation, unsupported terminals, or UI initialization failure.
    - `textual` requires the dashboard and fails before candidate execution if it cannot start.
    - `plain` preserves the current streaming line/heartbeat interface.
    - `--dry-run` remains orthogonal to UI selection.

    The sparse wrapper may pass validated terminal presentation variables such as `TERM` and `COLORTERM`; it must not broadly inherit the host environment.

    ## Execution and evidence architecture

    Introduce presentation-neutral events within `release_host_gate.py`:

    - `PlanDefined`
    - `TaskStarted`
    - `TaskHeartbeat`
    - `OutputReceived`
    - `TaskPassed`
    - `TaskWarned`
    - `TaskFailed`
    - `GateCompleted`

    The existing runner remains the sole subprocess owner. For every output or state transition it must:

    1. write and flush the authoritative evidence file;
    2. emit the corresponding presentation event.

    The dashboard must never parse terminal output to infer success. It renders structured runner events. A renderer failure must not convert passing candidate evidence into success or discard a candidate failure.

    Run blocking gate execution in a Textual thread worker. Deliver UI updates through Textual thread-safe messages or main-thread callbacks. Candidate subprocesses continue to receive closed stdin and exact argv without a shell.

    ## Dashboard layout

    ### Header

    Display:

    - release version;
    - abbreviated candidate commit;
    - dry-run or publication mode;
    - selected container runtime once known;
    - total elapsed time;
    - run-directory basename.

    ### Overall progress

    Use a determinate Textual progress bar measured in completed high-level tasks.

    - Do not derive percentage from elapsed time.
    - Do not show an ETA.
    - Before the plan is fully known, use an indeterminate bar.
    - After changed-path selection, set the final task total and mark unselected conditional checks as skipped.
    - The current task displays a spinner and its elapsed time.

    ### Task list

    Show a persistent scrollable list with these states:

    - pending;
    - running;
    - passed;
    - warning;
    - failed;
    - skipped.

    Rows show state icon, task label, and duration or skip reason. The plan covers:

    - immutable candidate validation;
    - isolated tool/runtime prerequisites;
    - locked Rust dependency fetch;
    - both Darwin builds;
    - native Darwin smoke;
    - foundation image build;
    - runtime image build;
    - local smoke alias preparation when required;
    - image inspection;
    - SBOM;
    - vulnerability policy;
    - canonical generated-container lifecycle;
    - each selected addon group;
    - LaTeX lifecycle;
    - rootless Podman readiness;
    - evidence manifest;
    - publication or dry-run completion.

    Selecting a row filters the visible log to that task. An explicit “All output” row restores the complete stream.

    ### Live-output box

    Use a read-only Textual `TextArea` with:

    - a visible border and title;
    - no syntax extra or tree-sitter dependency;
    - soft-wrap toggle;
    - automatic following while at the end;
    - pause-on-scroll or explicit follow toggle;
    - mouse and keyboard text selection;
    - `Ctrl+C` to copy selection;
    - `Ctrl+A` to select all within the focused log;
    - `End` to return to the live tail;
    - a binding to copy the entire currently selected task log;
    - a binding to reveal the authoritative evidence-log path.

    Batch programmatic appends to avoid repainting once per byte or partial fragment. Preserve complete output in evidence even if the UI later needs a bounded rendering optimization. Any UI truncation must be visible and must offer direct access to the full evidence file.

    ## Keyboard and lifecycle rules

    - `Tab` and `Shift+Tab`: move focus.
    - Arrow/PageUp/PageDown: navigate tasks or logs.
    - Space: toggle follow mode.
    - `w`: toggle wrapping.
    - `Ctrl+C`: copy selected text when the log has focus.
    - `?`: show key help.
    - `q`: close only after completion.
    - An abort action while running requires explicit confirmation and must terminate only the subprocess owned by the gate, wait for it, record failure evidence, and perform existing cleanup.
    - A terminal resize must preserve current task, selection, and follow state.

    ## Plain-renderer compatibility

    The plain renderer remains supported and testable. It must retain:

    - command lines;
    - live subprocess output;
    - ten-second quiet-command heartbeats;
    - running/passed/failed status lines;
    - actionable failure diagnostics;
    - final publication or dry-run message.

    Evidence contents and checksums must be identical for equivalent Textual and plain runs except for explicitly recorded presentation metadata.

    ## Security requirements

    - Textual receives no Docker, GitHub, SSH, Keychain, or publication credentials beyond what the existing gate process already holds.
    - Candidate-controlled text is rendered as literal text; Rich/Textual markup interpretation is disabled for raw command output.
    - OSC/control sequences from subprocesses are sanitized for display while raw bytes/text remain in evidence.
    - Log-copy features copy only user-selected or explicitly requested evidence text.
    - The UI does not introduce tmux, nested terminal sessions, shell command construction, or additional host mounts.
    - The trusted-control-path set includes every new lock artifact or changed wrapper that can select UI code or dependencies.

    ## Failure behavior

    - A failed task remains visible and selected.
    - The progress bar changes to failed styling but retains completed count.
    - The full captured diagnostic is shown in the log.
    - Cleanup tasks appear and execute even after failure.
    - Textual exits with the exact gate exit status.
    - Unexpected UI exceptions produce a concise diagnostic and, in `auto` mode only, allow the gate to continue with the plain renderer if candidate execution has not yet started. No automatic renderer restart occurs after candidate execution begins.

    ## Testing requirements

    Add coverage for:

    - UI-mode parsing and TTY/TERM auto-selection;
    - exact dependency pin and trusted lock;
    - task-state transitions and progress totals;
    - selected/skipped impact tasks;
    - output ordering: evidence flush precedes renderer event;
    - literal handling of ANSI, control sequences, and markup-like output;
    - copy/select/follow/wrap bindings;
    - failure and cleanup state rendering;
    - Textual headless snapshot or pilot tests at narrow and wide terminal sizes;
    - plain-renderer compatibility;
    - UI initialization failure fallback;
    - exit-code propagation;
    - no Textual dependency in the publisher.

    Tests must not start tmux or a container runtime merely to exercise the UI.

    ## Documentation requirements

    Update CONTRIBUTING and release maintenance documentation with:

    - the dashboard layout;
    - keyboard controls;
    - `--ui` modes;
    - plain-mode examples for captured logs and automation;
    - the distinction between presentation and signed evidence;
    - the exact evidence file to use when copying or attaching complete logs.

    ## Acceptance criteria

    Implementation is accepted when:

    1. all three requested UI surfaces are present and usable on the macOS host;
    2. log text can be selected and copied to another terminal window;
    3. a quiet command continuously shows active state and elapsed time;
    4. high-level progress remains truthful for mandatory, selected, and skipped tasks;
    5. plain mode produces the existing operator output and works without Textual rendering;
    6. evidence and publisher boundary tests remain clean;
    7. unit, E2E, integration, Clippy, formatting, Cargo audit, and shell contract tests pass;
    8. a fresh exact-candidate macOS `--dry-run` passes in Textual mode;
    9. the same prepared candidate completes a plain-mode dry-run or a bounded presentation-equivalence rehearsal without rebuilding publication artifacts;
    10. PR #373 records the exact successful candidate and evidence directory.
  type: insight
  state: captured
  review_due: '2026-08-19'
  tags:
  - release
  - textual
  - terminal-ui
  - issue-372
  - pr-373
  - security-boundary
  source: 'Owner-approved specification for PR #373'
---

# Textual release-host gate UI specification

## Status and relationships

- Status: accepted for implementation
- Decision: `DEC-20260812_1004-CordialFlute-use-textual-for-the-release-host`
- WorkItem: `BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted`
- GitHub: issue #372, PR #373
- Target entry point: `./scripts/maintain.sh release-host <run-dir> [--dry-run]`

## Objective

Make multi-minute macOS release-host validation visibly active, navigable, and copyable without changing its execution authority, evidence contract, or publication boundary.

The interactive terminal must provide:

1. a truthful overall progress bar;
2. a persistent high-level task list with state and elapsed time;
3. a bordered, scrollable live-output view whose text can be selected and copied between terminal windows.

## Source-layout constraint

Keep all production Python implementation in the existing `scripts/release_host_gate.py`.

That file owns:

- gate execution;
- the event and state model;
- the plain renderer;
- the Textual application;
- inline Textual CSS;
- task definitions and status reduction.

Do not add a second production Python module. Existing shell entry points and shell contract tests may change. A dependency lock file is permitted and does not violate the one-Python-file constraint. The publisher remains in its existing separate `scripts/release_host_publish.py` and must not import Textual.

## Dependency contract

- Pin Textual to an exact reviewed version and resolve all transitive packages in a committed lock artifact.
- The trusted `release-host-gate.sh` wrapper provisions that locked environment through the already approved uv executable and exact Python 3.14.6.
- Do not enable candidate-controlled PEP 723 metadata.
- Record Python, uv, Textual, and resolved dependency provenance in `evidence/darwin-build/toolchain.json`.
- Publication remains offline and Textual-free.

## Presentation selection

Support `--ui=auto|textual|plain` at the maintain and host-gate entry points.

- `auto` is the default.
- `auto` uses Textual only when stdin and stdout are suitable TTYs and `TERM` is not `dumb`.
- `auto` falls back to plain rendering for pipes, redirected output, automation, unsupported terminals, or UI initialization failure.
- `textual` requires the dashboard and fails before candidate execution if it cannot start.
- `plain` preserves the current streaming line/heartbeat interface.
- `--dry-run` remains orthogonal to UI selection.

The sparse wrapper may pass validated terminal presentation variables such as `TERM` and `COLORTERM`; it must not broadly inherit the host environment.

## Execution and evidence architecture

Introduce presentation-neutral events within `release_host_gate.py`:

- `PlanDefined`
- `TaskStarted`
- `TaskHeartbeat`
- `OutputReceived`
- `TaskPassed`
- `TaskWarned`
- `TaskFailed`
- `GateCompleted`

The existing runner remains the sole subprocess owner. For every output or state transition it must:

1. write and flush the authoritative evidence file;
2. emit the corresponding presentation event.

The dashboard must never parse terminal output to infer success. It renders structured runner events. A renderer failure must not convert passing candidate evidence into success or discard a candidate failure.

Run blocking gate execution in a Textual thread worker. Deliver UI updates through Textual thread-safe messages or main-thread callbacks. Candidate subprocesses continue to receive closed stdin and exact argv without a shell.

## Dashboard layout

### Header

Display:

- release version;
- abbreviated candidate commit;
- dry-run or publication mode;
- selected container runtime once known;
- total elapsed time;
- run-directory basename.

### Overall progress

Use a determinate Textual progress bar measured in completed high-level tasks.

- Do not derive percentage from elapsed time.
- Do not show an ETA.
- Before the plan is fully known, use an indeterminate bar.
- After changed-path selection, set the final task total and mark unselected conditional checks as skipped.
- The current task displays a spinner and its elapsed time.

### Task list

Show a persistent scrollable list with these states:

- pending;
- running;
- passed;
- warning;
- failed;
- skipped.

Rows show state icon, task label, and duration or skip reason. The plan covers:

- immutable candidate validation;
- isolated tool/runtime prerequisites;
- locked Rust dependency fetch;
- both Darwin builds;
- native Darwin smoke;
- foundation image build;
- runtime image build;
- local smoke alias preparation when required;
- image inspection;
- SBOM;
- vulnerability policy;
- canonical generated-container lifecycle;
- each selected addon group;
- LaTeX lifecycle;
- rootless Podman readiness;
- evidence manifest;
- publication or dry-run completion.

Selecting a row filters the visible log to that task. An explicit “All output” row restores the complete stream.

### Live-output box

Use a read-only Textual `TextArea` with:

- a visible border and title;
- no syntax extra or tree-sitter dependency;
- soft-wrap toggle;
- automatic following while at the end;
- pause-on-scroll or explicit follow toggle;
- mouse and keyboard text selection;
- `Ctrl+C` to copy selection;
- `Ctrl+A` to select all within the focused log;
- `End` to return to the live tail;
- a binding to copy the entire currently selected task log;
- a binding to reveal the authoritative evidence-log path.

Batch programmatic appends to avoid repainting once per byte or partial fragment. Preserve complete output in evidence even if the UI later needs a bounded rendering optimization. Any UI truncation must be visible and must offer direct access to the full evidence file.

## Keyboard and lifecycle rules

- `Tab` and `Shift+Tab`: move focus.
- Arrow/PageUp/PageDown: navigate tasks or logs.
- Space: toggle follow mode.
- `w`: toggle wrapping.
- `Ctrl+C`: copy selected text when the log has focus.
- `?`: show key help.
- `q`: close only after completion.
- An abort action while running requires explicit confirmation and must terminate only the subprocess owned by the gate, wait for it, record failure evidence, and perform existing cleanup.
- A terminal resize must preserve current task, selection, and follow state.

## Plain-renderer compatibility

The plain renderer remains supported and testable. It must retain:

- command lines;
- live subprocess output;
- ten-second quiet-command heartbeats;
- running/passed/failed status lines;
- actionable failure diagnostics;
- final publication or dry-run message.

Evidence contents and checksums must be identical for equivalent Textual and plain runs except for explicitly recorded presentation metadata.

## Security requirements

- Textual receives no Docker, GitHub, SSH, Keychain, or publication credentials beyond what the existing gate process already holds.
- Candidate-controlled text is rendered as literal text; Rich/Textual markup interpretation is disabled for raw command output.
- OSC/control sequences from subprocesses are sanitized for display while raw bytes/text remain in evidence.
- Log-copy features copy only user-selected or explicitly requested evidence text.
- The UI does not introduce tmux, nested terminal sessions, shell command construction, or additional host mounts.
- The trusted-control-path set includes every new lock artifact or changed wrapper that can select UI code or dependencies.

## Failure behavior

- A failed task remains visible and selected.
- The progress bar changes to failed styling but retains completed count.
- The full captured diagnostic is shown in the log.
- Cleanup tasks appear and execute even after failure.
- Textual exits with the exact gate exit status.
- Unexpected UI exceptions produce a concise diagnostic and, in `auto` mode only, allow the gate to continue with the plain renderer if candidate execution has not yet started. No automatic renderer restart occurs after candidate execution begins.

## Testing requirements

Add coverage for:

- UI-mode parsing and TTY/TERM auto-selection;
- exact dependency pin and trusted lock;
- task-state transitions and progress totals;
- selected/skipped impact tasks;
- output ordering: evidence flush precedes renderer event;
- literal handling of ANSI, control sequences, and markup-like output;
- copy/select/follow/wrap bindings;
- failure and cleanup state rendering;
- Textual headless snapshot or pilot tests at narrow and wide terminal sizes;
- plain-renderer compatibility;
- UI initialization failure fallback;
- exit-code propagation;
- no Textual dependency in the publisher.

Tests must not start tmux or a container runtime merely to exercise the UI.

## Documentation requirements

Update CONTRIBUTING and release maintenance documentation with:

- the dashboard layout;
- keyboard controls;
- `--ui` modes;
- plain-mode examples for captured logs and automation;
- the distinction between presentation and signed evidence;
- the exact evidence file to use when copying or attaching complete logs.

## Acceptance criteria

Implementation is accepted when:

1. all three requested UI surfaces are present and usable on the macOS host;
2. log text can be selected and copied to another terminal window;
3. a quiet command continuously shows active state and elapsed time;
4. high-level progress remains truthful for mandatory, selected, and skipped tasks;
5. plain mode produces the existing operator output and works without Textual rendering;
6. evidence and publisher boundary tests remain clean;
7. unit, E2E, integration, Clippy, formatting, Cargo audit, and shell contract tests pass;
8. a fresh exact-candidate macOS `--dry-run` passes in Textual mode;
9. the same prepared candidate completes a plain-mode dry-run or a bounded presentation-equivalence rehearsal without rebuilding publication artifacts;
10. PR #373 records the exact successful candidate and evidence directory.
