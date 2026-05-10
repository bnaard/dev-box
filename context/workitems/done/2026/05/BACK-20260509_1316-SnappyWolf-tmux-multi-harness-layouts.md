---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-SnappyWolf-tmux-multi-harness-layouts
  created: '2026-05-09T13:16:16+00:00'
  updated: '2026-05-09T22:19:13+00:00'
spec:
  title: 'tmux layouts: support multiple enabled harnesses with primary/secondary
    slot model'
  state: done
  type: task
  priority: high
  description: |
    ## Bug
    With `[ai.harness.claude]` and `[ai.harness.codex]` both `enabled = true`, `cli/src/tmux/layouts.rs` only spawns the first active provider (`providers.iter().find(|p| p.is_active())`). Second harness is never started in any layout's agent pane.

    ## Root cause
    `tmux_layout_script(layout, providers, ...)` resolves a single `provider.binary_name()` and templates it into the layout body. No fan-out for additional active harnesses.

    ## Greenfield proposal — proposed for v0.25.7 (awaiting owner approval)
    **1. New aibox.toml field per harness:** `order = N` (1 = primary, 2..N = secondaries; defaults to declaration order if unset).

    **2. Per-layout placement policy** (greenfield rewrite — not all current layouts make sense as-is):
    | Layout | 1 harness | N harnesses |
    |---|---|---|
    | `focus` | primary fullscreen | primary 80% top, secondaries split horizontally bottom 20% |
    | `ai` | yazi 20% \| primary 80% + windows {shell, git?} | yazi 15% \| primary 60% \| secondaries stacked vertically right 25% + windows {shell, git?} |
    | `dev` | yazi 50% \| primary 50% + git window | yazi 50% \| primary right top 60% / secondaries stacked right 40% + git window |
    | `cowork` | yazi top 30% / primary bottom 70% (degraded; cowork wants ≥2) | yazi top 25% / primary bottom-left 50% \| secondary bottom-right 50% (3+: secondaries stacked right) |
    | `cowork-swap` | yazi 20% \| primary 80% (degraded) | yazi 20% \| primary right top 50% / secondary right bottom 50% |
    | `browse` | yazi top 75% / primary bottom 25% | yazi top 70% / primary bottom-left 60% \| secondaries split horizontally bottom-right 40% |

    **3. Navigation (already bound — no new bindings needed):**
    - `prefix h/j/k/l` — pane navigation
    - `prefix f` — toggle pane zoom (NOTE: user mentioned "leader z b" — current binding is `f`. Keep `f` or remap to `z`? Decision needed.)

    **4. Alternatives — owner wants reconsidered (added 2026-05-09 by owner request):**

    Two extension paths to keep on the table during design, even though my initial proposal scoped them out:

    **4a. Inline layout DSL in aibox.toml.** Express splits/panes as a small TOML structure rather than a black-box layout name. Sketch:
    ```toml
    [customization.tmux.layouts.ai]
    windows = [
      { name = "ai", split = "h", panes = [
        { tool = "yazi", size = 20 },
        { harness = "primary", size = 60 },
        { split = "v", size = 20, panes = [ { harness = "secondary[*]" } ] },
      ]},
      { name = "shell", panes = [ { tool = "bash" } ] },
    ]
    ```
    Tradeoff: maximal flexibility without forking tmux.conf, but the DSL becomes a long-term maintenance contract (versioning, validation, docs). Risk of recreating tmuxinator/teamocil in TOML.

    **4b. User-defined layout files (drop-in).** `~/.config/tmux/layouts/<name>.sh` already exists as the rendered output target. Allow users to drop in their own `<name>.sh` files which `aibox-tmux-session.sh` picks up unchanged (current code already greps the script directory). aibox.toml just selects by name; if the file is user-authored, aibox doesn't overwrite it. Tradeoff: zero new schema, full power for advanced users, but no multi-harness fan-out integration unless the user-authored script reads `$AIBOX_HARNESSES` (or similar) env vars.

    **4c. (Original proposal — keep for v0.25.7 default behaviour).** Fixed named layouts with two knobs (`order`, `placement`). Smallest schema surface, fastest to ship, predictable behaviour for new users. Could coexist with 4a or 4b as the default, with 4a/4b as opt-in escape hatches.

    **Open design question:** ship 4c alone in v0.25.7, ship 4c + 4b in v0.25.7 (small additional surface — just a "don't overwrite if user-authored" check), or invest in 4a as a v0.25.8/0.26.0 epic? Owner has flagged 4a/4b as worth considering rather than rejecting.

    ## Implementation outline
    - `cli/src/tmux/layouts.rs`: change signature to take all active providers in order; render N panes per layout.
    - `cli/src/config.rs`: add `order: Option<u32>` to `[ai.harness.<name>]` schema; resolve at config load.
    - Snapshot tests per layout × {1, 2, 3+ harnesses}.
    - Migration entity (BR-CLEANUP-ARCH item 6 / Variant 3 emission).

    ## Acceptance criteria
    - 2 harnesses enabled → both spawn in all six layouts per the table above.
    - Primary slot honors lowest `order` value; ties broken by declaration order.
    - Existing single-harness behavior preserved when only one harness is enabled (no regression).
    - `aibox doctor` passes; `tmux_layouts_start_expected_windows_and_panes` and the multi-harness snapshot variants pass.

    ## Open question for owner
    - Keep `prefix f` zoom or remap to `prefix z`?
    - For `focus`-with-multiple-harnesses, is "primary 80% / stacked secondaries 20%" the right policy, or should `focus` lock to a single harness regardless of how many are enabled?

    ## References
    - `cli/src/tmux/layouts.rs:16-21` — single-provider resolution
    - `aibox.toml:401-444` — current `[ai.harness.*]` schema
    - DEC-20260508_2115-SilentFern (status-format slot order — analog precedent for how to scope the schema decision)
  started_at: '2026-05-09T22:18:30+00:00'
  completed_at: '2026-05-09T22:19:13+00:00'
---

## Transition note (2026-05-09T22:19:13+00:00)

4c+4b scope shipped. Commit 917f160 + merge e427e62. AIBOX_LAYOUT_AGENT_SPLIT/RATIO env knobs + drop-in support; 16/16 cargo tests pass incl. 3 new.
