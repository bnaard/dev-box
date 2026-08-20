# Migrations Index

## Pending (0)

None.

## In Progress (0)

None.

## Applied (6)

| Date       | Migration                                | Notes |
|------------|------------------------------------------|-------|
| 2026-07-20 | MIG-20260720_1350-ContentSync-processkit-content-sync — processkit v0.27.4 → v0.27.5 | 0 changed upstream, 0 conflicts, 17 new, 1 removed, 0 stale-removed (5 groups affected) |
| 2026-07-20 | MIG-20260720_1408-SnappyComet-canonicalize-note-spec-key-order-for — local-project  → | Canonicalize Note spec key order for v0.27.5 doctor |
| 2026-07-22 | MIG-20260722_1623-ContentSync-processkit-content-sync — processkit v0.27.5 → v0.28.1 | 0 changed upstream, 0 conflicts, 5 new, 16 removed, 0 stale-removed (6 groups affected) |
| 2026-07-26 | MIG-20260726_1903-ContentSync-processkit-content-sync — processkit v0.28.3 → v0.28.4 | 0 changed upstream, 0 conflicts, 722 new, 0 removed, 0 stale-removed (44 groups affected) |
| 2026-08-01 | MIG-20260731_1857-ContentSync-processkit-content-sync — processkit v0.28.4 → v0.28.5 | 0 changed upstream, 1 conflicts, 0 new, 0 removed, 0 stale-removed (1 groups affected) |
| 2026-08-20 | MIG-20260820_0727-RuntimeSync-aibox-runtime — aibox-runtime-home 0.33.2 → 0.34.0 | 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected) |

## Rejected (3)

| Date       | Migration                                | Reason |
|------------|------------------------------------------|--------|
| 2026-07-17 | MIG-20260717_1220-RuntimeSync-aibox-runtime — aibox-runtime-home 0.27.6 → 0.27.5 | Malformed no-op runtime sync: reports zero changed/conflicted/new/removed files and a backwards version transition from… |
| 2026-07-20 | MIG-20260720_1404-AgileBridge-normalize-legacy-note-frontmatter-for-v0 — local-project  → | Rejected after diagnosis: the Note is schema-valid; pk-doctor's naive frontmatter delimiter split truncates YAML block … |
| 2026-07-20 | MIG-20260720_1407-FineForge-normalize-markdown-rule-in-note-yaml — local-project  → | Rejected after refinement: the first triple-hyphen occurs inside a Markdown table separator, so changing the content wo… |
