---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0726-HappyFjord-implement-dec-trueclover-per-layout-multi
  created: '2026-05-10T07:26:27+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    depends-decision: DEC-TrueClover
  updated: '2026-05-10T07:48:45+00:00'
spec:
  title: 'Implement DEC-TrueClover: per-layout multi-harness behaviour for browse
    / cowork / cowork-swap / dev / focus'
  state: done
  type: task
  priority: high
  description: |
    ## Background

    DEC-20260510_0346-TrueClover is now **accepted** by the owner. SmartLark (BACK-20260510_0336-SmartLark) shipped multi-harness handling for the `ai` layout only. The other 5 layouts still silently ignore order-2+ harnesses (the original SnappyWolf bug remains there).

    ## Goal

    Implement the accepted per-layout multi-harness behaviour:

    | Layout | Behaviour when ≥2 harnesses are active |
    |---|---|
    | browse | Hide AI panes entirely (browse = file-focused). Order-1 also hidden. |
    | cowork | Order-1 in claude column; secondaries stacked hidden in same column; cycle with `prefix j/k`. |
    | cowork-swap | Mirror of cowork (vertical layout instead of horizontal). |
    | dev | Order-1 in claude side column; secondaries tabbed in same column; cycle with `prefix j/k`. |
    | focus | Single visible harness (order-1 by default); `prefix j/k` switches the visible one. |

    ## Implementation hints

    - All layouts live in `cli/src/tmux/layouts.rs`. Pattern from SmartLark's `ai_secondary_panes` is the reference; reuse / generalize where possible.
    - "Hide" semantics for browse / cowork / cowork-swap: use tmux's `select-pane -d` (disable) or split-window then immediately resize to 0 — verify what the existing 4c knobs do and reuse.
    - "Tabbed" semantics for dev: each secondary harness becomes a tmux **window** within the layout, switchable via `prefix [num]` or with the explicit `prefix j/k` aliases. Caveat: this conflicts with the "tools = windows" pattern from the sibling WorkItem — **see sibling for coordination on naming/numbering**.
    - Order resolution stays the same: stable list order from `[ai].harnesses` in aibox.toml, or explicit `[ai.harness.<name>] order = N`.
    - `prefix j` / `prefix k` keybindings are already added by SmartLark; reuse them but constrain to "only cycle within harness panes/windows", not all panes.

    ## Acceptance

    - For each of the 5 layouts, with 2+ harnesses enabled, behaviour matches the table above.
    - Existing single-harness layouts unchanged (no regression).
    - `prefix j/k` cycles correctly within the multi-harness scope per layout (panes for cowork/-swap/dev/focus; not applicable for browse).
    - Unit tests cover the order-resolution + per-layout split logic.
    - Cargo tests pass; new tests cover the 5 layouts.

    ## Refs

    - DEC-20260510_0346-TrueClover (accepted)
    - BACK-20260510_0336-SmartLark (ai layout — reference implementation)
    - DEC-20260509_2125-CoolFrog / DEC-20260508_2115-SilentFern (slot-order discipline — not applicable here, but pattern of "code change + paired DEC" applies)
    - Sibling WorkItem: tools-as-windows generalization (window-numbering coordination)
  started_at: '2026-05-10T07:48:17+00:00'
  completed_at: '2026-05-10T07:48:45+00:00'
---

## Transition note (2026-05-10T07:48:45+00:00)

Implemented and merged in commit 42a5b5c + merge 1c3c885. Per-layout multi-harness behaviour for browse (hidden), cowork/cowork-swap (stacked hidden panes via select-pane -d, prefix j/k cycle), dev/focus (secondary harnesses as named windows). New helpers cowork_secondary_panes / dev_secondary_windows / focus_secondary_windows in cli/src/tmux/layouts.rs. 15 new tests pass. DEC-20260510_0346-TrueClover satisfied.
