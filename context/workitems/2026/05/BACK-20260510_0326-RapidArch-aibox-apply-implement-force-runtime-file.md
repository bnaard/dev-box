---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0326-RapidArch-aibox-apply-implement-force-runtime-file
  created: '2026-05-10T03:26:00+00:00'
  labels:
    version: v0.25.7-followup
    area: runtime-sync
spec:
  title: 'aibox apply: implement --force-runtime-file &lt;path&gt; flag (referenced
    by Variant 3 migration body)'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Background

    GentleFern's Variant 3 Migration emission (commit 1898a31) writes a per-file recommendation table. The default `review-manually` recommendation references a `--force-runtime-file <path>` flag in its remediation hint — but the flag does not yet exist in `aibox apply`.

    Users following the migration's instructions today will hit "unknown flag".

    ## Goal

    Implement `aibox apply --force-runtime-file <path>` (and likely `--force-runtime-file-all` for bulk). The flag should:

    1. Accept one or more file paths under `.aibox-home/`.
    2. For each: bypass the classifier and overwrite-with-canonical-current.
    3. Log each forced overwrite with the migration ID that triggered the recommendation.
    4. Refuse paths outside `.aibox-home/` and refuse to descend into symlinks.

    ## Acceptance

    - `aibox apply --force-runtime-file ~/.aibox-home/.config/yazi/yazi.toml` overwrites that one file with current canonical, leaves all others to normal classification.
    - A unit test exercises the bypass path.
    - The Variant 3 migration body's "How to apply" section now correctly resolves to a runnable command.

    ## Refs

    - Commit 1898a31 (GentleFern)
    - File: cli/src/runtime_sync.rs (where the migration body is generated)
    - Sibling WorkItem: Variant 3 recommendation engine
---
