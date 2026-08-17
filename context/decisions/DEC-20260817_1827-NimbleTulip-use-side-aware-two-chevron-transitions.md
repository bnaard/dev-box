---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260817_1827-NimbleTulip-use-side-aware-two-chevron-transitions
  created: '2026-08-17T18:27:01+00:00'
  updated: '2026-08-17T18:28:28+00:00'
spec:
  title: Use side-aware two-chevron transitions for PowerKit plugin spacing
  state: accepted
  decision: Patch tmux-powerkit plugin spacing so left-aligned plugins render previous
    segment to status gap to next segment with right-facing chevrons, and right-aligned
    plugins render the mirrored color orientation with left-facing chevrons. Do not
    add a literal rectangular gap cell. Use this as the v0.x aibox default.
  context: The initial compact-spacing decision removed or replaced individual gap
    cells, but live screenshots showed fused segments, doubled glyphs, or mixed chevron
    and rectangular boundaries. Upstream research showed that plugin spacing hardcodes
    one color orientation even though separator direction varies by status side, while
    the window renderer handles both directions correctly.
  rationale: This mirrors PowerKit's own window-transition algorithm, passes deterministic
    left/right color-orientation assertions, passes an isolated tmux/asciinema visual
    test, and was explicitly approved after live inspection.
  alternatives:
  - option: Disable PowerKit element spacing
    rejected_because: Segments visually fuse with no breathing room.
  - option: Use closing chevron plus a literal blank cell
    rejected_because: Spacing is too wide and mixes arrow and rectangular gap shapes.
  - option: Remove one chevron without correcting side-aware colors
    rejected_because: Produces doubled artifacts or inconsistent boundaries depending
      on status side.
  consequences: aibox carries a small fail-closed patch against the pinned tmux-powerkit
    renderer until upstream fixes the side-aware plugin-spacing branch. Focused tests
    must cover left and right raw format invariants and retain an isolated tmux/asciinema
    visual artifact.
  decided_at: '2026-08-17T18:27:01+00:00'
  supersedes: DEC-20260817_1615-ValiantWren-use-one-cell-compact-gaps-between
---
