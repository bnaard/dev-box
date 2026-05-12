---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-RUNTIME-DRIFT-20260512T044714
  created: 2026-05-12 04:47:14+00:00
  updated: '2026-05-12T05:02:47+00:00'
spec:
  source: aibox-runtime-drift
  source_url: aibox://runtime-drift
  to_version: 0.25.9
  variant: 3
  state: applied
  generated_by: aibox apply
  generated_at: 2026-05-12 04:47:14+00:00
  summary: 5 drifted managed runtime file(s) found at 0.25.9
  started_at: '2026-05-12T05:02:46+00:00'
  applied_at: '2026-05-12T05:02:47+00:00'
  progress_notes:
  - timestamp: '2026-05-12T05:02:47+00:00'
    actor: mcp
    note: Resolved during pk-doctor cleanup by preserving existing local managed runtime
      edits as intentional current workspace state; no canonical overwrite performed.
---

# Migration MIG-RUNTIME-DRIFT-20260512T044714

## Drifted managed runtime files (Variant 3 — BR-CLEANUP-ARCH item 6)

The following managed `.aibox-home/` runtime files have been modified on the host and match **neither** the current canonical aibox generation **nor** any known archived template snapshot. They may represent intentional user customisations.

`aibox apply` left these files **untouched**. Review each one and decide whether to preserve the local edit or restore the canonical generated content.

## Per-file recommendations

| file | reason-for-classification | recommendation |
|------|--------------------------|----------------|
| `.aibox-home/.vim/vimrc` | content matches neither current canonical nor any archived snapshot | review-manually |
| `.aibox-home/.config/git/config` | content matches neither current canonical nor any archived snapshot | review-manually |
| `.aibox-home/.config/yazi/plugins/git.yazi/main.lua` | content matches neither current canonical nor any archived snapshot | review-manually |
| `.aibox-home/.config/cheatsheet.txt` | content matches neither current canonical nor any archived snapshot | review-manually |
| `.aibox-home/.claude.json` | content matches neither current canonical nor any archived snapshot | review-manually |

## How to resolve

For each file above, choose one of:
- **`preserve-as-is`** — keep your local edit; mark this migration applied.
- **`overwrite-with-canonical`** — run `aibox apply --force-runtime-file <path>` (once that flag ships) or manually copy the canonical content from `context/templates/aibox-home/<version>/<path>`.
- **`review-manually`** — compare live vs canonical using your diff tool and cherry-pick the parts you want to keep.
