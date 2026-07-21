---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260721_1306-ActiveLeaf-expose-forge-github-hosts-through-tmux
  created: '2026-07-21T13:06:23+00:00'
spec:
  title: Expose Forge GitHub hosts through tmux status configuration
  state: accepted
  decision: Add customization.tmux.status.forge.github-hosts as the persistent aibox
    configuration for exact GitHub host and SSH-alias matching, defaulting to github.com,
    and render it into the managed PowerKit tmux options.
  rationale: The plugin-level option alone is not usable persistently in derived projects
    because aibox apply regenerates tmux configuration. A first-class status-plugin
    setting keeps aibox.toml authoritative and preserves exact-match behavior.
  alternatives:
  - option: Require manual tmux set-option commands
    reason_rejected: The setting disappears with tmux/runtime regeneration and is
      not reproducible.
  - option: Treat every github-* hostname as GitHub
    reason_rejected: Prefix matching can misclassify unrelated hosts and weakens the
      exact-host safety boundary.
  consequences: The config schema, commented serialization, tmux renderer, documentation,
    and tests must support and validate the host list. Delivery requires a new CLI
    and runtime image release before derived projects can use it.
  decided_at: '2026-07-21T13:06:23+00:00'
---
