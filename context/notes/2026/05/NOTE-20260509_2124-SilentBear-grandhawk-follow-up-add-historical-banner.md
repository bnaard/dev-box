---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2124-SilentBear-grandhawk-follow-up-add-historical-banner
  created: '2026-05-09T21:24:44+00:00'
spec:
  title: 'GrandHawk follow-up: add historical banner to NOTE-SureSwan v0.23.2 handover'
  body: |
    ## Background

    `context/notes/2026/05/NOTE-20260503_1104-SureSwan-aibox-v0-23-2-release-handover.md` has frontmatter `apiVersion: processkit.projectious.work/v2` — MCP-owned. Five Zellij hits in the body, but two are immutable historical strings (a real commit message and an obsolete-but-frozen backlog WorkItem id) that **must not be rewritten**.

    ## Hit map

    | Line | Excerpt | Treatment |
    |------|---------|-----------|
    | 20 / 94 | `` `908cf2f fix: improve runtime cleanup and zellij status controls` `` | KEEP verbatim — real commit message; rewriting would be revisionist. |
    | 64 / 138 | `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin` ("build native Zellij plugin") | KEEP id verbatim; WorkItem now obsolete post-v0.25.x tmux migration but its id is immutable. |
    | 72 / 146 | "Pick up the native Zellij plugin/keybar backlog item when ready" | Stale — superseded by tmux/NobleCrane work. |

    ## Recommended treatment: historical-artifact banner

    Insert after the title line `# Session Handover: aibox v0.23.2 Release`, in both `spec.body` (line ~10) and the rendered body (line ~84):

    ```
    > **Historical** — this is a v0.23.2 (May 2026) release-handover snapshot. References
    > to Zellij and `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin`
    > describe the pre-v0.25.x multiplexer; aibox migrated to tmux in v0.25.x (NobleCrane).
    > Commit hashes and backlog ids preserved verbatim for audit. Kept as released-state record.
    ```

    Do not edit the commit-message string or the backlog-item id slug. They are frozen historical facts.

    ## Action

    Apply via the MCP path that supports v2-frontmatter note edits (or hand-apply once such a path exists). Until then this fleeting note records the pending edit.

    ## Provenance

    Originally written by the T3 GrandHawk agent as `context/migrations/pending/grandhawk-sureswan.md` because the sandbox blocked `/tmp/` writes. That misfiled propose-only file has been promoted to this Note and the original deleted.
  type: fleeting
  state: captured
  review_due: '2026-05-16'
  tags:
  - grandhawk
  - zellij-sweep
  - follow-up
  - v2-frontmatter
  - historical-artifact
  source: T3 docs-cleanup agent (BACK-20260508_2320-GrandHawk)
---

## Background

`context/notes/2026/05/NOTE-20260503_1104-SureSwan-aibox-v0-23-2-release-handover.md` has frontmatter `apiVersion: processkit.projectious.work/v2` — MCP-owned. Five Zellij hits in the body, but two are immutable historical strings (a real commit message and an obsolete-but-frozen backlog WorkItem id) that **must not be rewritten**.

## Hit map

| Line | Excerpt | Treatment |
|------|---------|-----------|
| 20 / 94 | `` `908cf2f fix: improve runtime cleanup and zellij status controls` `` | KEEP verbatim — real commit message; rewriting would be revisionist. |
| 64 / 138 | `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin` ("build native Zellij plugin") | KEEP id verbatim; WorkItem now obsolete post-v0.25.x tmux migration but its id is immutable. |
| 72 / 146 | "Pick up the native Zellij plugin/keybar backlog item when ready" | Stale — superseded by tmux/NobleCrane work. |

## Recommended treatment: historical-artifact banner

Insert after the title line `# Session Handover: aibox v0.23.2 Release`, in both `spec.body` (line ~10) and the rendered body (line ~84):

```
> **Historical** — this is a v0.23.2 (May 2026) release-handover snapshot. References
> to Zellij and `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin`
> describe the pre-v0.25.x multiplexer; aibox migrated to tmux in v0.25.x (NobleCrane).
> Commit hashes and backlog ids preserved verbatim for audit. Kept as released-state record.
```

Do not edit the commit-message string or the backlog-item id slug. They are frozen historical facts.

## Action

Apply via the MCP path that supports v2-frontmatter note edits (or hand-apply once such a path exists). Until then this fleeting note records the pending edit.

## Provenance

Originally written by the T3 GrandHawk agent as `context/migrations/pending/grandhawk-sureswan.md` because the sandbox blocked `/tmp/` writes. That misfiled propose-only file has been promoted to this Note and the original deleted.
