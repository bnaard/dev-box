---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260429_1135-BraveBison-implement-aibox-owned-harness
  created: '2026-04-29T11:35:08+00:00'
spec:
  title: Implement aibox-owned harness and runtime scaffolding fixes
  state: accepted
  decision: 'Treat the accepted bugfix plan as aibox-owned implementation work: aibox will generate stable harness permission/scaffold files, stop all compose services on `aibox stop`, scaffold generated gitignore entries for aibox/processkit runtime artifacts, and keep processkit owning the semantic skill-gate contract and marker format.'
  context: The user accepted a plan covering Claude/other harness MCP permission prompts, companion container stop behavior, repeated skill-gate hook failures in derived projects, `.agents` and `context/.state` gitignore policy, and `.aibox-local.env` scaffolding. The changes cross aibox runtime generation, harness integration, and processkit-installed runtime state.
  rationale: aibox owns reproducible project scaffolding and harness adapters, while processkit owns skills, MCP server definitions, and the skill-gate contract. Keeping the implementation boundary explicit avoids duplicating processkit logic in aibox while letting aibox make derived projects usable without repeated harness prompts or leaked local credentials.
  alternatives:
  - option: Move all permission and gitignore policy to processkit
    reason_rejected: processkit does not own aibox-generated harness config files or derived project container scaffolding.
  - option: Only document manual user fixes
    reason_rejected: The reported failures recur in derived projects and should be fixed in generated output, not left as repetitive setup work.
  consequences: Requires harness-specific permission emitters and regression coverage in aibox. Derived projects should get stable generated ignores and environment-file references after `aibox sync`.
  decided_at: '2026-04-29T11:35:08+00:00'
---
