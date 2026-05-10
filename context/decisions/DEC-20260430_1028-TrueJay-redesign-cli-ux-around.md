---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260430_1028-TrueJay-redesign-cli-ux-around
  created: '2026-04-30T10:28:00+00:00'
spec:
  title: Redesign CLI UX around verb-resource grammar
  state: accepted
  decision: 'Pursue a breaking CLI UX redesign centered on a small, predictable verb/resource
    grammar: init, apply, up, down, get, describe, set, edit, reset, delete, doctor,
    and self. Keep reset as an explicit scoped verb for project/runtime/theme safety
    rather than hiding all destructive behavior under delete.'
  context: The current aibox CLI has a strong core workflow but a sprawling top-level
    command surface and mixed mutation semantics. The owner asked for a radical redesign
    without backward-compatibility constraints, with consistency and predictability
    similar to kubectl.
  rationale: A verb/resource grammar reduces command overhead while making new capabilities
    predictable. Keeping reset scoped preserves important UX clarity for destructive
    recovery workflows such as project reset while still avoiding ambiguous top-level
    remove/reset/uninstall overlap.
  alternatives:
  - option: Keep current command taxonomy and add aliases only
    status: rejected
    reason: Lower migration cost but does not address top-level command sprawl or
      inconsistent mutation behavior.
  - option: Use pure kubectl-style delete for all destructive operations
    status: rejected
    reason: Systematic, but loses the user-recognizable reset intent and makes project
      reset look like ordinary deletion.
  consequences: Requires a breaking release, full docs rewrite, command help redesign,
    test updates, and migration guidance. Existing command names may be removed or
    temporarily preserved as hidden aliases depending on release policy.
  decided_at: '2026-04-30T10:28:00+00:00'
---
