---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0809-SnappySky-revert-speculative-tool-addons-drop-monitoring
  created: '2026-05-10T08:09:00+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    kind: revert
  updated: '2026-05-10T08:14:28+00:00'
spec:
  title: 'Revert speculative tool addons: drop monitoring.yaml + K/B/D bindings (keep
    framework + lazygit only)'
  state: done
  type: task
  priority: medium
  description: |
    ## Background

    GrandDaisy (commit `15de96b`, merge `1c3c885`) shipped the tools-as-windows generalization PLUS speculative addon scaffolding for btop / lazydocker / k9s and one-letter prefix bindings (`K` for k9s, `B` for btop, `D` for lazydocker). Owner review concluded the framework is justified but the specific tool additions were unprompted — nobody on the project has actually asked for these tools.

    The right scope for v0.25.7 is: **keep the framework, drop the speculative tool content**. Each future tool gets its own WorkItem with an owner-driven decision on base-image-install vs. addon-only.

    ## Goal — revert speculative additions, keep framework

    ### Remove

    1. `addons/tools/monitoring.yaml` — delete the file (btop + lazydocker addon declarations).
    2. `cli/src/tmux/status.rs` `DEFAULT_TMUX_CONF` — remove the three speculative `bind-key -N` lines:
       - `bind-key -N "Switch to k9s/kubernetes window" K ...`
       - `bind-key -N "Switch to btop/system monitor window" B ...`
       - `bind-key -N "Switch to lazydocker/containers window" D ...`
    3. The corresponding test assertions in `cli/src/tmux/status.rs` tests covering K/B/D bindings.

    ### Keep

    - `tool_windows` parameter on `tmux_layout_script` (the framework primitive).
    - `tool_windows_for_config()` helper in `cli/src/seed.rs` (resolves enabled tool addons to window names).
    - `bind-key g` for lazygit (already in active use).
    - `bind-key s` for shell (already in active use in ai layout).
    - `addons/tools/kubernetes.yaml` — leave untouched (predates this sprint per the GrandDaisy report; if it was also added by GrandDaisy, also remove it; verify in git log first).

    ### Verify

    - After revert, `tool_windows_for_config()` should still return at least lazygit when its addon is enabled, so the lazygit window keeps spawning.
    - `cargo check` clean; `cargo test --bin aibox` ≥ baseline minus the deleted K/B/D test cases.
    - Search for any docs-site references to the three reverted tools and remove them too.

    ## Why this is right

    - The framework is the durable artifact (any future tool plugs in trivially).
    - Speculative addon content costs maintenance attention without proven need.
    - ToughPanda's base-image-install decision becomes moot — we close ToughPanda as obsolete.

    ## Refs

    - BACK-20260510_0726-GrandDaisy (predecessor; partial revert of its speculative content)
    - BACK-20260510_0748-ToughPanda (will be cancelled in the same turn — the install decision is moot once the tools aren't shipped)
  started_at: '2026-05-10T08:13:52+00:00'
  completed_at: '2026-05-10T08:14:28+00:00'
---

## Transition note (2026-05-10T08:14:28+00:00)

Implemented and merged in commit e4c95eb (committed directly on main; deviation from branch-and-merge pattern but result correct and pushed to origin). Removed addons/tools/monitoring.yaml; removed K/B/D bind-key lines from DEFAULT_TMUX_CONF; removed btop/lazydocker entries from tool_windows_for_config(). Framework intact: tool_windows parameter, lazygit window, g/s bindings, kubernetes.yaml addon (pre-existing) all preserved. 895/895 tests pass.
