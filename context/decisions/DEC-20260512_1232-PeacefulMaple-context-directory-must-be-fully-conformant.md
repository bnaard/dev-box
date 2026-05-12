---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260512_1232-PeacefulMaple-context-directory-must-be-fully-conformant
  created: '2026-05-12T12:32:51+00:00'
  updated: '2026-05-12T12:33:04+00:00'
spec:
  title: context directory must be fully conformant, including historical entities
  state: accepted
  decision: The project requires a 100% conformant `context/` directory tree. Historical filename timestamp mismatches, placeholder timestamps, old applied migration filenames, legacy CLI migration briefings, schema-invalid historical notes/logs/bindings, and missing vocabulary declarations must be transformed or migrated into the current canonical schema and storage layout. pk-doctor must not resolve these findings by downgrading them to tolerated or non-actionable historical exceptions.
  context: The previous decision `DEC-20260512_1228-CalmBison-pk-doctor-grandfathers-historical-processkit-context` was recorded incorrectly during pk-doctor cleanup. The owner immediately rejected that direction and clarified that historical context should not be grandfathered; it should be transformed until the tree is fully conformant.
  rationale: A fully conformant context tree gives processkit and downstream aibox users one operational standard. Tolerating historical exceptions keeps pk-doctor noisy or weakens it as a hard gate. The correct cleanup path is migration/transformation of historical data, with schemas and doctor checks remaining strict enough to enforce the canonical state.
  alternatives:
  - option: Grandfather historical entities and downgrade findings to informational/non-actionable
    rejected_because: Explicitly rejected by the owner; it preserves historical drift instead of producing a conformant tree.
  - option: Disable or relax pk-doctor checks for historical paths
    rejected_because: This weakens the gate and hides the exact class of drift the owner wants eliminated.
  - option: Transform historical entities and storage layout to current policy
    rejected_because: Accepted despite higher migration effort because it produces the desired canonical tree.
  consequences: The cleanup work must rename or migrate historical files, repair schema-invalid specs, move legacy briefings into canonical storage or archive form, and update schemas only for legitimate canonical vocabulary. Future pk-doctor findings of this class should be treated as migration work, not accepted policy exceptions.
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  decided_at: '2026-05-12T12:32:51+00:00'
  supersedes: DEC-20260512_1228-CalmBison-pk-doctor-grandfathers-historical-processkit-context
---
