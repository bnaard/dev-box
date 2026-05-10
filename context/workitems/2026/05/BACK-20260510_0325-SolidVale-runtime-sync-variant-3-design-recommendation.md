---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0325-SolidVale-runtime-sync-variant-3-design-recommendation
  created: '2026-05-10T03:25:51+00:00'
  labels:
    version: v0.25.8-candidate
    area: runtime-sync
    needs-decision: 'true'
spec:
  title: 'runtime_sync Variant 3: design recommendation engine to promote ''review-manually''
    to auto-resolve'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Background

    GentleFern's BR-CLEANUP item 6 (commit 1898a31, merge 55a12cc) shipped Variant 3 Migration emission for drifted-but-not-historical runtime files. The emitted Migration body lists each file with a default recommendation of `review-manually`.

    This always-defer default is the safe baseline but misses cases where aibox could confidently auto-resolve. We need a recommendation engine that examines each drifted file and decides one of: `review-manually`, `overwrite-with-canonical`, or `preserve-as-is`.

    ## Heuristic candidates

    - **File age:** if the user-edited version is older than N days and aibox has shipped multiple intermediate canonical generations since, lean `overwrite-with-canonical`.
    - **Edit scope:** if the diff is purely whitespace/comment, `overwrite-with-canonical`. If the diff touches semantic lines, `review-manually`.
    - **User markers:** if the file contains an `# aibox: keep` (or similar) marker, `preserve-as-is`. If `# aibox: regenerate`, `overwrite-with-canonical`.
    - **File category:** treat config files (yaml, toml) more conservatively than scripts.

    ## Decision needed

    This is more architectural than implementation — should pair a DecisionRecord capturing the heuristic and the marker syntax before code lands.

    ## Refs

    - Commit 1898a31 (GentleFern Variant 3 emission)
    - File: cli/src/runtime_sync.rs (drift_migration_document writer)
    - Sibling WorkItem: --force-runtime-file flag (referenced in Variant 3 migration body but not yet implemented)
---
