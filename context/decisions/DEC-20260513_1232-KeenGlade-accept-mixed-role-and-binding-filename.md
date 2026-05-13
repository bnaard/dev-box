---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260513_1232-KeenGlade-accept-mixed-role-and-binding-filename
  created: '2026-05-13T12:32:22+00:00'
  updated: '2026-05-13T12:56:25+00:00'
spec:
  title: Accept mixed role and binding filename policies during v0.26.3 transition
  state: superseded
  decision: Accept the temporary mixed filename policy for Role and Binding entities
    while v0.26.3 introduces deterministic role and binding IDs alongside older timestamped
    entities.
  context: pk-doctor reports storage.filename-policy-mixed for context/roles and context/bindings
    plus context_hygiene binding.filename-style-mixed after v0.26.3 integration.
  rationale: The mixed filenames are caused by upstream v0.26.3 adding deterministic
    role and binding catalog entities while the existing dogfood context still contains
    historical timestamped entities. Renaming existing entities is higher risk than
    the cosmetic warning during this integration pass, and no processkit migration
    tool exists yet to normalize these IDs safely.
  consequences: The warnings should be treated as an accepted policy exception until
    processkit ships a dedicated filename-normalization migration. Entity IDs remain
    stable and existing references are not broken.
  deciders:
  - TEAMMEMBER-cora
  decided_at: '2026-05-13T12:32:22+00:00'
  superseded_by: DEC-20260513_1249-GrandSpruce-strictly-migrate-processkit-context-instead-of
---
