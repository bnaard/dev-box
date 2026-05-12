---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260510_0346-TrueClover-per-layout-multi-harness-default-geometry
  created: '2026-05-10T03:46:39+00:00'
  updated: '2026-05-10T07:26:01+00:00'
spec:
  title: Per-layout multi-harness default geometry for browse / cowork / cowork-swap / dev / focus
  state: accepted
  decision: "Adopt the following per-layout multi-harness behaviours when \u22652 harnesses are active, pending owner sign-off before implementation: browse \u2014 no change (AI panes hidden); cowork \u2014 secondary harnesses stacked in the AI column (hidden, cycled with leader j/k); cowork-swap \u2014 mirror of cowork; dev \u2014 secondary harnesses tabbed in the AI side column; focus \u2014 leader j/k switches the visible harness."
  context: SmartLark (BACK-20260510_0336-SmartLark) shipped the ai-layout multi-harness fix and the leader j/k/z keybindings. The WorkItem requires pairing a DecisionRecord for the other 5 layouts before touching them. The per-layout proposals come directly from the WorkItem spec table. The ai layout is already implemented; the other 5 layouts are proposed here for owner review.
  rationale: Pairing a DecisionRecord before implementing per-layout changes ensures owner review of geometry proposals that involve trade-offs between screen real estate and discoverability. The ai layout was owner-specified; the others are more ambiguous and need explicit sign-off before implementation.
  alternatives:
  - option: Implement all 5 layouts immediately without owner review
    pros: Faster delivery
    cons: Risk of mismatching owner expectations on geometry trade-offs
  - option: Skip non-ai layouts entirely
    pros: Smallest diff, minimum blast radius
    cons: Multi-harness behaviour remains undefined for browse/cowork/dev/focus users
  related_workitems:
  - BACK-20260510_0336-SmartLark-tmux-layouts-multi-harness-ai-layout
  decided_at: '2026-05-10T07:26:01+00:00'
---
