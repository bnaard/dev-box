---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260504_1219-CoolPeak-scope-generated-runtime-finalization
  created: '2026-05-04T12:19:49+00:00'
spec:
  title: Scope Generated Runtime Finalization
  state: accepted
  decision: Keep `aibox apply --no-container` as a real sync path, and add a narrower
    generated-runtime finalization path for release/self-maintenance that writes only
    repo-owned generated runtime surfaces needed for drift cleanup. The scoped path
    may update `.devcontainer/*`, `aibox.lock`, `context/migrations/*.md`, and `context/templates/aibox-home/<version>/`;
    it must not write provider or harness configuration such as `.codex/*`, `.claude/*`,
    MCP registrations, hook registrations, or preauth files.
  context: 'Patch releases left repo-owned generated runtime surfaces behind after
    the host phase, while `aibox apply --no-container` is intentionally broad: it
    can regenerate devcontainer files, runtime snapshots, processkit content, MCP
    registration, hook files, preauth data, migrations, and lock state. Using that
    broad command as a release cleanup step risks overwriting provider or harness
    files such as `.codex/hooks.json`. The e2e companion remains the right place to
    validate real container/runtime behavior, including Zellij plugin status and Yazi
    editor integration.'
  rationale: The release process needs an idempotent way to commit intended generated
    drift after host-side assets are available, but provider-specific configuration
    belongs to the consuming workspace and must not be touched by a release-finalization
    helper. Separating these surfaces makes the command useful for self-release automation
    and keeps accidental config mutation out of the workflow.
  alternatives:
  - option: Use `aibox apply --no-container` directly in release automation
    status: rejected
    reason: It writes too many surfaces and can mutate provider/harness configuration.
  - option: Make `apply --no-container` a dry projection command
    status: rejected
    reason: It would change an existing sync contract and surprise callers that expect
      generated files and lock state to be written.
  - option: Rely on manual post-release git cleanup
    status: rejected
    reason: Manual cleanup is error-prone and already allowed generated runtime drift
      to accumulate.
  consequences: Release automation can add a final repo-side commit for generated
    runtime surfaces without running the full sync pipeline. Existing users who rely
    on `apply --no-container` keep its current behavior. E2E tests should cover runtime-visible
    failures rather than treating generated-file diffs as sufficient proof of Zellij/Yazi
    behavior. Future generator changes need to choose explicitly whether they belong
    to scoped generated-runtime output or full apply/sync output.
  decided_at: '2026-05-04T12:19:49+00:00'
---
