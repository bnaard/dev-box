---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260817_1615-ValiantWren-use-one-cell-compact-gaps-between
  created: '2026-08-17T16:15:40+00:00'
  updated: '2026-08-17T18:28:28+00:00'
spec:
  title: Use one-cell compact gaps between PowerKit status elements
  state: superseded
  decision: Keep a single separator-width gap between PowerKit plugin elements by
    retaining the closing separator cell and removing the additional literal blank
    cell. Make this the standard spacing for v0.x aibox-generated tmux deployments.
  context: The generated v0.x tmux status used PowerKit elements-spacing=both. Its
    two-cell separator-plus-blank gaps made right-side header elements appear roughly
    twice as far apart as the compact first-row tmux window tabs. Disabling spacing
    entirely joined elements too tightly.
  rationale: The one-cell live trial visually matched the compact tab row more closely
    and the owner explicitly selected it after comparing the existing two-cell spacing
    and the zero-gap trial.
  alternatives:
  - option: Keep PowerKit elements-spacing=both unchanged
    rejected_because: The separator plus literal blank produces a visually oversized
      two-cell gap.
  - option: Disable plugin spacing
    rejected_because: Adjacent elements have no visible gap and were judged too tight.
  consequences: The aibox PowerKit integration needs a maintained compact-spacing
    renderer adjustment and regression coverage. Generated v0.x tmux configurations
    retain spacing mode while rendered gaps consume one cell instead of two.
  decided_at: '2026-08-17T16:15:40+00:00'
  superseded_by: DEC-20260817_1827-NimbleTulip-use-side-aware-two-chevron-transitions
---
