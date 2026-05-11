---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1200-HappyFern-use-harness-order-for-tmux-harness
  created: '2026-05-11T12:00:26+00:00'
spec:
  title: Use harness_order for tmux harness placement semantics
  state: accepted
  decision: aibox will expose [ai].harness_order as the explicit ordering control
    for tmux layout placement. The terms 1st harness, 2nd harness, and 3rd harness
    refer to enabled harnesses after applying harness_order; enabled harnesses missing
    from harness_order are appended in canonical order.
  context: Per-harness [ai.harness.<name>] tables are not ordered because they deserialize
    into a map. Existing tmux layouts already depend on ordered harness semantics,
    but the current visible config does not expose a clear ordering field.
  rationale: An explicit order list is clearer than numeric weights for layout placement
    and preserves per-harness enabled/install/version controls. It creates stable
    vocabulary for the upcoming layout rework.
  consequences: Config loading must sort the effective enabled harness list by harness_order
    before layout generation. Documentation/comments and tests should describe 1st/2nd/3rd
    harness semantics in terms of harness_order.
  decided_at: '2026-05-11T12:00:26+00:00'
---
