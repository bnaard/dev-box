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
  title: 'aibox apply: implement --force-runtime-file &lt;path&gt; flag (referenced by Variant 3 migration body)'
  state: backlog
  type: task
  priority: medium
  description: "## Background\n\nGentleFern's Variant 3 Migration emission (commit 1898a31) writes a per-file recommendation table. The default `review-manually` recommendation references a `--force-runtime-file <path>` flag in its remediation hint \u2014 but the flag does not yet exist in `aibox apply`.\n\nUsers following the migration's instructions today will hit \"unknown flag\".\n\n## Goal\n\nImplement `aibox apply --force-runtime-file <path>` (and likely `--force-runtime-file-all` for bulk). The flag should:\n\n1. Accept one or more file paths under `.aibox-home/`.\n2. For each: bypass the classifier and overwrite-with-canonical-current.\n3. Log each forced overwrite with the migration ID that triggered the recommendation.\n4. Refuse paths outside `.aibox-home/` and refuse to descend into symlinks.\n\n## Acceptance\n\n- `aibox apply --force-runtime-file ~/.aibox-home/.config/yazi/yazi.toml` overwrites that one file with current canonical, leaves all others to normal classification.\n\
    - A unit test exercises the bypass path.\n- The Variant 3 migration body's \"How to apply\" section now correctly resolves to a runnable command.\n\n## Refs\n\n- Commit 1898a31 (GentleFern)\n- File: cli/src/runtime_sync.rs (where the migration body is generated)\n- Sibling WorkItem: Variant 3 recommendation engine"
---
