---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260504_0920-DeepCliff-approve-runtime-ui-doctor-and-e2e
  created: '2026-05-04T09:20:01+00:00'
spec:
  title: Approve runtime UI, doctor, and E2E hardening plan
  state: accepted
  decision: Implement the approved plan to fix lazygit-disabled Zellij layout leakage,
    make Zellij status rendering testable and visible, extend pk-doctor with runtime-template/theme/aibox.toml
    schema drift checks, and broaden E2E/asciinema tests for scaffolding, updates,
    migrations, reset, and visual regressions.
  context: The owner reported lazygit tabs opening despite lazygit being disabled,
    native Zellij status rows rendering as black/blank lines, missing pk-doctor checks
    for runtime theme/template and aibox.toml schema drift, and insufficient E2E/asciinema
    coverage for realistic aibox workflows.
  rationale: The existing tests cover some generated files and visual theme colors,
    but they miss stale runtime sync, status-line content, reset/migration lifecycle
    drift, and full scaffold-update-reset workflows. The plan preserves aibox.toml
    as the declarative source and catches regressions through fast hermetic tests
    plus focused full-container/asciinema tests.
  alternatives:
  - option: Only fix reported bugs
    reason_rejected: Would leave the same regression class uncovered in E2E and doctor
      diagnostics.
  - option: Only add full-container visual tests
    reason_rejected: Too slow for most regressions and would miss cheap deterministic
      generation/schema failures.
  consequences: Implementation will add focused product fixes and tests across seed/runtime
    generation, doctor diagnostics, plugin rendering, E2E reset/migration flows, and
    visual recordings. Work is split into at most two parallel implementation lanes
    to keep ownership clear.
  related_workitems:
  - BACK-20260504_0919-QuietThorn-strengthen-runtime-doctor-e2e-coverage
  decided_at: '2026-05-04T09:20:01+00:00'
---
