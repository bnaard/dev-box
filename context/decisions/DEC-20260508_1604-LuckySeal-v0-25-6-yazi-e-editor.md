---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_1604-LuckySeal-v0-25-6-yazi-e-editor
  created: '2026-05-08T16:04:33+00:00'
  updated: '2026-05-08T16:04:42+00:00'
spec:
  title: 'v0.25.6: Yazi ''e'' editor handoff hard-cut — remove persistent vim pane,
    on-demand popup'
  state: accepted
  decision: For v0.25.6 we eliminate the persistent vim/editor pane from every layout
    (`dev`, `ai`, `focus`, `cowork`, `cowork-swap`, `browse`). Yazi `e` is rebound
    to open a full-screen tmux `display-popup -E` running vim on the marked file;
    `:q` closes the popup and returns focus to yazi. Yazi `Enter` keeps the existing
    suspending-launcher behavior (vim takes over the yazi pane until `:q`). The 128-line
    `open-in-editor.sh` discovery machinery (`find_editor_pane`, directional fallback,
    `vim-loop` detection, send-keys to long-lived vim) is deleted. Owner-approved.
  context: Owner reports the persistent-vim-pane handoff has been visibly broken in
    some form across more than 20 aibox releases — the yazi `e` keybinding loads files
    into the loop-running vim screen unreliably, with the regression appearing, getting
    fixed, and re-appearing. The 2026-05-08 cross-cutting review (DEC-20260508_1515-SilentAsh)
    did not include this scope. Owner now requests a hard cut as part of v0.25.6.
  rationale: 'The fragility is structural: reliably reusing a long-lived vim across
    tmux pane discovery, command sends, and pane-id tracking depends on (a) the pane
    title staying `editor`/`vim-loop`, (b) vim not being mid-modal-state when `:edit`
    is sent, (c) the discovery script finding the right pane across layout-specific
    topologies, (d) tmux send-keys not racing with shell history. Each of these has
    its own failure mode and they compound. A stateless full-screen popup eliminates
    the entire surface — every `e` press starts a fresh vim, no discovery, no send-keys,
    no shared state. The cost (vim startup latency, no shared register/buffer state
    across opens) is low for a code/config editing workflow where files are typically
    opened one at a time. `:q` gracefully closes the popup via `display-popup -E`.
    Owner explicitly chose this trade-off after living with the recurring regression.'
  alternatives:
  - option: Yet another rewrite of the discovery + send-keys logic
    rejected_because: Same architectural pattern; will regress again. Owner has rejected
      this path after 20+ releases of the cycle.
  - option: Keep persistent vim pane but use tmux pipe-pane with named pipes for file-open
      command
    rejected_because: Adds a new failure surface (pipe lifecycle, race with vim's
      own input handling) without removing the old one. Marginal robustness gain.
  - option: Drop the popup approach in favor of a new tmux window per file
    rejected_because: Pollutes the window list; doesn't match owner's spec ('full
      screen pane on the current screen').
  - option: Make Enter and 'e' identical (both suspend yazi)
    rejected_because: Owner explicitly wants the two paths to differ — Enter is the
      launcher pattern, 'e' is the explicit overlay for marked files. Differentiation
      has UX value.
  consequences: |
    POSITIVE:
    - 'e' becomes 100% reliable — no shared state to break.
    - ~128 lines of `open-in-editor.sh` collapse to ~15.
    - All layouts get simpler (one less pane each); cognitive load and tmux-config size drop.
    - Vim startup overhead per file open is acceptable (modern vim ~50ms cold).
    - Eliminates the pane-discovery code path that has been the source of every yazi/vim regression in this area.

    NEGATIVE / RISKS:
    - Vim register/buffer state is no longer shared across files opened with 'e' (each is its own session). For users who relied on this, Enter still gives a single-session experience.
    - Layout regen is a visible change — release notes must call it out.
    - Existing e2e tests asserting the persistent vim pane will need to be updated/removed.
  deciders:
  - TEAMMEMBER-cora
  decided_at: '2026-05-08T16:04:33+00:00'
  related_workitems:
  - BACK-20260508_1604-GrandWillow-yazi-vim-pane-hard-cut
---
