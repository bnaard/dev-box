---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0336-SmartLark-tmux-layouts-multi-harness-ai-layout
  created: '2026-05-10T03:36:30+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    needs-decision: 'true'
  updated: '2026-05-10T07:14:33+00:00'
spec:
  title: 'tmux layouts: multi-harness ai-layout default geometry + leader keybindings
    + per-layout multi-harness proposals'
  state: done
  type: bug
  priority: high
  description: |
    ## Background

    SnappyWolf (commit `917f160`, merge `e427e62`) shipped the 4c+4b mechanism (env-var knobs + drop-in support). Open items not addressed:

    1. The **default geometry** for ai layout when ≥2 harnesses are enabled.
    2. **Leader keybindings** for switching between harness panes and zooming.
    3. **Per-layout multi-harness defaults** for browse / cowork / cowork-swap / dev / focus.

    The original symptom (only one harness starts in ai screen when two are enabled) needs verification post-fix.

    ## Owner-specified ai layout default

    When ≥2 harnesses are enabled in `aibox.toml [ai].harnesses`:
    - The **order-1 harness** (first in the list, or via an explicit `order =` field) gets the **main pane at ~80% width**.
    - Subsequent harnesses (order 2, 3, …) render as **small stacked panes** beneath or beside the main pane.
    - Switch between harness panes: **leader j / leader k** (next / prev).
    - Zoom toggle for the focused pane: **leader z** (tmux-native `prefix z` is already a toggle — verify naming and document; possibly add `leader b` as an explicit "back to non-zoom" alias if the toggle semantics confuse users).

    ## Per-layout proposals (initial — refine in DecisionRecord)

    | Layout | Multi-harness behaviour |
    |---|---|
    | **ai** | Order-1 at 80% main; others stacked small (per owner spec). |
    | **browse** | Hide all AI panes (focus is yazi/code preview). Multi-harness irrelevant here. |
    | **cowork** | Order-1 AI on left half; user vim on right half. Secondary harnesses → tabbed/zoomable side stack of width 0%, only visible on `leader z`. |
    | **cowork-swap** | Mirror of cowork — order-1 AI on right half; user vim on left. Secondary stack same as cowork. |
    | **dev** | User vim primary; AI in side pane; secondary harnesses tabbed in the same side pane (cycle with `leader j/k`). |
    | **focus** | Single pane, hide everything except the focused harness. `leader j/k` switches the visible harness. |

    These are proposals — file a DecisionRecord and refine with the owner before implementing.

    ## Implementation hints

    - Source-of-truth: `images/base-debian/config/tmux/layouts/{ai,browse,cowork,cowork-swap,dev,focus}.sh` (or wherever seed.rs renders them from).
    - Honor the existing 4c knobs (`AIBOX_LAYOUT_AGENT_SPLIT`, `AIBOX_LAYOUT_AGENT_RATIO`) — extend, don't redesign.
    - Order resolution: respect explicit `[ai.harness.<name>] order = N` if present in aibox.toml; otherwise stable list order.
    - Keybindings go in `images/base-debian/config/tmux/tmux.conf` (binding section).

    ## Acceptance

    - With 2 harnesses enabled (e.g. claude + codex), `aibox apply` + tmux session start → both panes render on the ai layout per the owner spec.
    - `leader j/k` cycles focus between harness panes.
    - `leader z` zooms toggle works.
    - The 5 other layouts handle multi-harness per the agreed defaults (after DecisionRecord pass).
    - Unit tests cover order resolution.
    - DecisionRecord paired before merge.

    ## Refs

    - BACK-20260509_1316-SnappyWolf (predecessor; partial fix)
    - DEC-20260509_2125-CoolFrog (slot-order contract — separate, but relevant as precedent for paired DEC pattern)
  started_at: '2026-05-10T07:14:07+00:00'
  completed_at: '2026-05-10T07:14:33+00:00'
---

## Transition note (2026-05-10T07:14:33+00:00)

Implemented and merged in commit 3d2d8d6 + merge ded4dd3. ai_secondary_panes() in cli/src/tmux/layouts.rs cascades split-window for harnesses 2..N (primary at ~80%); leader z explicit zoom binding added; leader j/k descriptions clarified to 'next/prev harness pane'. 5 new tests pass. Per-layout multi-harness defaults for browse/cowork/cowork-swap/dev/focus parked in DEC-20260510_0346-TrueClover (proposed); pending owner acceptance for follow-up implementation.
