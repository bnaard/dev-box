---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime
  created: '2026-05-07T14:47:39+00:00'
spec:
  title: Remove Zellij And Rebuild aibox Runtime On tmux
  state: accepted
  decision: aibox will remove Zellij completely from the generated runtime stack and
    redesign its terminal runtime around tmux as the sole multiplexer. The redesign
    will integrate a curated tmux baseline, including TPM, tmux-sensible, tmux-powerkit,
    vim-tmux-navigator, tmux-continuum, tmux-yank, and tmux-resurrect, while preserving
    aibox's declarative aibox.toml model and downstream reproducibility.
  context: 'The previous backlog item was to evaluate tmux as an alternative or fallback
    after repeated Zellij runtime incidents, including plugin/server CPU blast radius
    and status rendering instability. The owner has now made the architecture decision:
    no fallback evaluation, no partial dual-stack transition as the target state,
    and no continued Zellij ownership in generated runtimes. This is a major runtime
    redesign that affects images, generated .aibox-home templates, CLI generation,
    docs, tests, release notes, and migration behavior for derived projects.'
  rationale: tmux has a simpler and more externally observable status/plugin model
    for aibox's needs, with a mature plugin ecosystem for sensible defaults, navigation,
    session persistence, clipboard/yank behavior, and status customization. Removing
    Zellij rather than carrying both multiplexers reduces runtime complexity, eliminates
    the Zellij WASM/plugin permission surface, and keeps the generated runtime easier
    to reason about after several Zellij-specific incidents.
  alternatives:
  - option: Keep Zellij default and harden plugin path
    status: rejected
    reason: Recent incidents show Zellij plugin/server blast radius is too large for
      the default runtime surface, and further hardening would still leave a complex
      default path.
  - option: Offer tmux as optional fallback while keeping Zellij
    status: rejected
    reason: Dual multiplexer support would increase generator, docs, tests, and support
      matrix complexity while preserving the unstable Zellij path.
  - option: Disable all multiplexer functionality
    status: rejected
    reason: aibox still needs a reproducible multi-pane AI-ready terminal workspace;
      tmux provides that with lower runtime risk.
  consequences: This is a breaking/runtime-major change. aibox must remove Zellij
    binaries, layouts, plugins, permission caches, status WASM artifacts, Zellij docs,
    and Zellij-specific tests. It must introduce tmux image dependencies, generated
    tmux config, plugin installation/pinning strategy, runtime startup/attach behavior,
    status-line design, migrations from Zellij-oriented generated home content, updated
    E2E/visual smoke coverage, and release notes that tell downstream users how to
    adapt. The old tmux-evaluation backlog item should be superseded by implementation
    workitems rather than treated as open research.
  related_workitems:
  - BACK-20260507_1341-CalmEagle-evaluate-tmux-runtime-fallback
  - BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign
  - BACK-20260505_2222-KeenHare-investigate-zellij-status-plugin-errors
  decided_at: '2026-05-07T14:47:39+00:00'
---
