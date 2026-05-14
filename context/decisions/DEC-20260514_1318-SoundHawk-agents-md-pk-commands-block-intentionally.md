---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260514_1318-SoundHawk-agents-md-pk-commands-block-intentionally
  created: '2026-05-14T13:18:51+00:00'
spec:
  title: AGENTS.md pk-commands block intentionally diverges from processkit template
  state: accepted
  decision: 'Project records an explicit exception for the AGENTS.md pk-managed:pk-commands
    block: aibox''s build/test/lint/fmt commands target a Rust+Cargo toolchain (`cd
    cli && cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`,
    `cargo fmt`), not the processkit upstream template''s Node/docs-site commands
    (`npm --prefix docs-site run build`, `uv run scripts/smoke-test-servers.py`).
    The `pk-doctor` `agents_md_hygiene.managed-block-drift` finding for this block
    is therefore expected and acceptable.'
  context: pk-doctor v0.26.7's `agents_md_hygiene` check compares the SHA256 of each
    `pk-managed:*` block against the bundled processkit template (`context/templates/processkit/v0.26.7/AGENTS.md`).
    The pk-commands block must, by design, hold project-specific build commands —
    but the check has no opt-out flag and warns on any divergence. The check's documented
    escape hatch is "unless the project records an explicit exception" — this DEC
    is that exception.
  rationale: aibox is a Rust CLI; the upstream processkit template ships Node/Python
    commands because processkit itself is a docs+Python project. Reverting aibox's
    pk-commands block to the template values would break /pk-build, /pk-test, /pk-lint
    adapters across the project. The drift is intrinsic to the dual-purpose nature
    of the pk-commands block (managed marker, but content is consumer-specific).
  alternatives:
  - option: Replace block with template commands
    rejected_because: would break Rust toolchain command adapters
  - option: Remove managed-block markers around pk-commands
    rejected_because: would re-trigger managed-block-missing warning
  - option: File an upstream issue requesting a per-project commands schema
    rejected_because: useful future work, but does not unblock the current session
  consequences: pk-doctor will continue to surface `agents_md_hygiene.managed-block-drift`
    for pk-commands until upstream processkit ships a per-project command-override
    knob (e.g. a `pk-commands` schema validation that ignores hash drift). Treat it
    as accepted-policy noise.
  decided_at: '2026-05-14T13:18:51+00:00'
---
