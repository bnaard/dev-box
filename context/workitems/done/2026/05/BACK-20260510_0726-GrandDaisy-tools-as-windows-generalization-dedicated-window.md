---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0726-GrandDaisy-tools-as-windows-generalization-dedicated-window
  created: '2026-05-10T07:26:48+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    area-2: addons
  updated: '2026-05-10T07:48:50+00:00'
spec:
  title: 'Tools-as-windows generalization: dedicated window per enabled tool addon
    (lazygit / k9s / btop / lazydocker)'
  state: done
  type: task
  priority: medium
  description: |
    ## Background

    aibox already wires lazygit as a separate tmux window when its addon is enabled (`include_lazygit_tab` boolean, `cli/src/tmux/layouts.rs`). The pattern works well — adding a tool doesn't perturb layout geometry, and the window list in line1-left of the statusline (SilentFjord) automatically surfaces it.

    This WorkItem generalizes the pattern to additional tools that are common in aibox-style developer workflows: **k9s** (kubernetes), **btop** (system monitor), **lazydocker** (containers). Each gets its own window when its addon is enabled.

    ## Goal

    1. Extend the layout script generator to add a window per enabled tool addon, in addition to the existing layout windows (and lazygit window).
    2. Add explicit one-letter `prefix` keybindings for fast access:

    | Binding | Window |
    |---|---|
    | `prefix g` | git (lazygit) — already wired |
    | `prefix K` | k9s (capital K to avoid collision with `prefix-k = prev-pane`) |
    | `prefix B` | btop |
    | `prefix D` | lazydocker |
    | `prefix s` | shell (already exists in ai layout) |

    3. Make tool selection driven by `aibox.toml` `[addons]` table — same pattern as docs addons. A tool addon being declared and enabled triggers window creation; absence means the window is skipped.

    ## Implementation hints

    - Add tool addons to `cli/src/addons/` alongside the existing `docs.rs` and `python.yaml` patterns. Each addon yaml should declare:
      - default version
      - install step (apt / curl / etc. — match existing patterns)
      - `tmux_window: true` (new boolean for the layout generator)
    - Layout generator (`cli/src/tmux/layouts.rs`) reads enabled tool addons and emits one `tmux new-window -n <name>` per tool, after the layout body, before the existing lazygit window.
    - Keybindings live in `images/base-debian/config/tmux/tmux.conf` (binding section). Use `bind-key -N` notation for help-key visibility (consistent with existing bindings).
    - `prefix 0` should switch to the primary layout window; existing `prefix [num]` (number-by-position) keeps working.

    ## Coordination with sibling WorkItem

    The DEC-TrueClover implementation also uses windows for the `dev` layout's tabbed-secondaries pattern. Coordinate:
    - Layout windows reserve numbers 0–9 (primary layout window = 0; secondaries 1, 2, …).
    - Tool windows take consecutive numbers AFTER layout windows (so they don't collide with layout-internal windows).
    - Naming convention: layout window = layout name (`dev`, `ai`, …); tool window = tool name (`git`, `k9s`, `btop`, `lazydocker`); shell window = `shell`.

    ## Acceptance

    - With each tool addon enabled in `aibox.toml`, `aibox apply` creates a corresponding tmux window after layout setup.
    - `prefix g/K/B/D/s` keybindings switch to the named windows (or are silently no-op if the window doesn't exist).
    - Existing single-tool flows (lazygit only) keep working without changes.
    - Unit tests cover: tool addon detection, window emission, keybinding rendering.
    - Statusline (line1-left, SilentFjord) lists every active window correctly.

    ## Refs

    - Existing pattern: lazygit window in `cli/src/tmux/layouts.rs` (`include_lazygit` boolean)
    - Sibling WorkItem: TrueClover implementation (window-numbering coordination)
    - Ownership: layout = `cli/src/tmux/layouts.rs`; addons = `cli/src/addons/`; bindings = `images/base-debian/config/tmux/tmux.conf`
  started_at: '2026-05-10T07:48:18+00:00'
  completed_at: '2026-05-10T07:48:50+00:00'
---

## Transition note (2026-05-10T07:48:50+00:00)

Implemented and merged in commit 15de96b + merge 1c3c885. Dedicated tmux window per enabled tool addon (lazygit/k9s/btop/lazydocker), `tool_windows` parameter threaded through tmux_layout_script. New addon yaml addons/tools/monitoring.yaml (btop apt + lazydocker GitHub releases, default-disabled); k9s already in addons/tools/kubernetes.yaml. Five prefix bindings (`g`/`K`/`B`/`D`/`s`) added to DEFAULT_TMUX_CONF. 894/895 tests pass. Base-image install follow-up filed as BACK-20260510_0748-ToughPanda.
