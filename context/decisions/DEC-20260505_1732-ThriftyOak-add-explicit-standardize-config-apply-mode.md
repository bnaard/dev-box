---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260505_1732-ThriftyOak-add-explicit-standardize-config-apply-mode
  created: '2026-05-05T17:32:18+00:00'
spec:
  title: Add explicit standardize config apply mode
  state: accepted
  decision: Add an opt-in `aibox apply --standardize-config` mode that rewrites `aibox.toml` into the current canonical grouped, commented schema while keeping normal `aibox apply` conservative.
  context: Derived projects can accumulate old generated structure, deprecated fields, and poor section ordering. Normal apply should avoid surprise destructive rewrites, but users need a deliberate way to recreate a standard config while preserving recognized project settings.
  rationale: An explicit flag makes the destructive-looking behavior discoverable and intentional. It lets aibox parse and migrate recognized current settings, emit the standard renderer output, and drop obsolete generated surfaces without making routine apply noisy or risky.
  alternatives:
  - option: Make every `aibox apply` harshly rewrite config
    assessment: Rejected because it would surprise derived projects and generate large diffs during routine reconciliation.
  - option: Add a separate `aibox config standardize` command
    assessment: Deferred because the operation is part of apply-time reconciliation and benefits from existing migration ordering.
  consequences: Normal apply remains conservative. Users can choose a larger canonical rewrite when upgrading older projects. Unknown schema keys should still block rather than be silently discarded, so users get a warning/error instead of data loss.
  decided_at: '2026-05-05T17:32:18+00:00'
---
