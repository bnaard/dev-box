---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260512_1228-CalmBison-pk-doctor-grandfathers-historical-processkit-context
  created: '2026-05-12T12:28:55+00:00'
  updated: '2026-05-12T12:33:04+00:00'
spec:
  title: pk-doctor grandfathers historical processkit context while enforcing active contracts
  state: superseded
  decision: pk-doctor should not fail the repository for immutable or explicitly grandfathered historical processkit context. Historical filename timestamp mismatches, old applied migration filenames, legacy CLI migration briefings, placeholder IDs from the initial import window, and append-only logs missing fields introduced later are treated as informational/non-actionable. Active contract validation remains strict for parse errors, current schema violations not covered by a grandfather rule, and new vocabulary values not declared in schemas.
  context: 'The user asked to resolve pending migrations and all pk-doctor errors, warnings, and actionable infos. The current findings were dominated by historical context imported before the v2 schema and storage policies settled: old filename dates, applied migration filename suffixes, logs without actor, legacy note body placement, mixed binding/role filename policies, and missing schema vocabulary entries for already-used event/type/kind values. Bulk-renaming or mutating append-only historical entities would create churn and conflict with the event-log immutability model.'
  rationale: The durable fix is to encode the compatibility boundary in schemas and doctor checks. Doctor should distinguish active defects from grandfathered historical state. Schema vocabularies should include legitimate historical values that still exist in v2 context, while hygiene checks should surface historical layout facts as non-actionable information instead of blocking every session.
  alternatives:
  - option: Bulk rewrite all historical context files to match the newest filename, storage, and schema policies
    rejected_because: This would hand-edit many historical entities, including append-only logs, and create high-risk churn for little operational value.
  - option: Leave pk-doctor noisy and rely on human interpretation
    rejected_because: The user explicitly asked for pk-doctor errors, warnings, and actionable infos to be resolved; recurring noise weakens the release gate.
  - option: Disable the affected pk-doctor checks entirely
    rejected_because: That would hide real future defects. The chosen approach keeps checks active while carving out documented historical compatibility.
  consequences: 'pk-doctor becomes a cleaner release/session gate: current defects still fail, while grandfathered historical artifacts are reported as non-actionable INFO or skipped where immutable. Future new vocabulary values still need schema updates, and future storage policy changes should update doctor with explicit compatibility rules rather than producing permanent WARN noise.'
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  decided_at: '2026-05-12T12:28:55+00:00'
  superseded_by: DEC-20260512_1232-PeacefulMaple-context-directory-must-be-fully-conformant
---
