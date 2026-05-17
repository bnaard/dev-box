---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260517_1027-TenderMoon-use-unified-ai-execution-policy-axes
  created: '2026-05-17T10:27:03+00:00'
spec:
  title: Use unified AI execution policy axes with per-harness mapping
  state: accepted
  decision: 'aibox.toml will expose AI harness execution policy through stable aibox
    vocabulary: filesystem, approval, and network. Global defaults live under [ai.execution],
    and per-harness overrides live under [ai.harness.<name>.execution]. MCP permission
    config remains separate and must not expose harness-specific implementation fields
    such as Codex trust_level or Claude mode as the canonical user model.'
  context: Accepted after the v0.26.6 release exposed that Codex workspace-write remounts
    .git read-only in its command sandbox, breaking normal git operations inside an
    already-isolated aibox devcontainer.
  rationale: The filesystem, approval, and network axes express user intent across
    harnesses without pretending that all harnesses implement identical sandbox machinery.
    Codex can map filesystem=container-full to sandbox_mode="danger-full-access" to
    avoid read-only .git inside trusted devcontainers, while weaker harnesses can
    map unsupported axes as best-effort warnings or no-ops.
  consequences: Config generation must add parser/schema/defaults, generated aibox.toml
    docs, Codex config projection, and tests. Existing MCP permission settings remain
    focused on MCP tool allow/deny patterns.
  related_workitems:
  - BACK-20260517_1026-StableOtter-ai-execution-policy-schema
  decided_at: '2026-05-17T10:27:03+00:00'
---
