---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260505_1231-TallDawn-represent-ai-harness-selection-under-ai
  created: '2026-05-05T12:31:14+00:00'
spec:
  title: Represent AI harness selection under ai config
  state: accepted
  decision: Move user-facing AI harness and model-provider selection in aibox.toml under the [ai] namespace. AI harnesses remain installable and removable, but their public configuration is no longer represented as addon selection; addon YAMLs continue as internal install/build recipes where useful.
  context: 'The current config exposes AI in two places: [ai] for harness/provider intent and addon-style sections for provider CLI installation. The user accepted consolidating this because all AI-related intent is semantically one domain, while installation/deinstallation should remain available.'
  rationale: Keeping AI harness selection under [ai] makes aibox.toml easier to reason about and avoids treating provider CLIs as ordinary tool bundles. Internally resolving [ai] choices through existing addon machinery preserves reuse of package recipes, version handling, and deinstallation behavior without leaking the implementation model into user config.
  alternatives:
  - option: Keep AI harnesses as normal addons
    status: rejected
    reason: It preserves implementation simplicity but keeps AI configuration split across unrelated sections.
  - option: Duplicate AI harnesses in both [ai] and [addons] permanently
    status: rejected
    reason: It creates two sources of truth and invites conflicting configuration.
  - option: Remove addon recipes for AI harnesses entirely
    status: rejected
    reason: It would throw away useful install/deinstall machinery; the better boundary is public semantic config over internal recipe resolution.
  consequences: Scaffolded config should place AI harness selection and per-harness install options under [ai]. The config loader should accept legacy addon-facing AI configuration for compatibility and merge it into the new model with a deprecation path. Documentation and examples should describe addons as generic tool bundles and AI harnesses as [ai]-owned selections.
  decided_at: '2026-05-05T12:31:14+00:00'
---
