---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260514_1902-ShinyLake-fetch-latest-image-version-doesn-t
  created: '2026-05-14T19:02:51+00:00'
spec:
  title: fetch_latest_image_version doesn't follow GHCR tag-list pagination
  state: backlog
  type: bug
  priority: medium
  description: |
    ## Symptom

    After v0.26.2 (Phase 1 + Phase 2 complete; `base-debian-v0.26.0`, `…v0.26.2`, and `…latest` all confirmed live in GHCR), `aibox apply` still emits:

    ```
    ==> Resolved aibox image 'latest' → v0.25.12
    ```

    …instead of `→ v0.26.2`. The `verify_release_images_in_ghcr()` Bash guard (shipped in v0.26.1) is unaffected because it uses direct `buildx imagetools inspect --raw <tag>` per-tag — but `cli/src/update.rs::fetch_latest_image_version` queries `/v2/projectious-work/aibox/tags/list` and only reads page 1.

    ## Root cause

    `update.rs` issues a single `ureq::get("https://ghcr.io/v2/projectious-work/aibox/tags/list")` and parses the response. Docker Registry v2 returns paginated tag lists by default (~100 tags per page) with a `Link: <next-url>; rel="next"` header. The current code doesn't follow that header, so any tag created after GHCR's default page-1 cutoff is invisible to the resolver.

    Reproduction: GHCR currently has 107 `base-debian-*` tags total. `?n=200` returns the full list with v0.26.2 at the top; the bare endpoint cuts off around v0.25.12, which is exactly what `aibox apply` reports.

    ## Fix

    In `cli/src/update.rs::fetch_latest_image_version`:

    1. Parse the `Link` header from each response (Docker Registry sends `</v2/...?last=...&n=...>; rel="next"`).
    2. Follow `rel="next"` until exhausted; concatenate the `tags` arrays.
    3. Then run the existing semver max-pick.

    Alternatively, request a larger page size via `?n=1000` — Docker Registry honors `n` up to the server-configured maximum. Simpler but less robust to future growth.

    ## Acceptance

    - A fresh `aibox apply` (with `[container.image].version = "latest"` and no other changes) resolves to the highest semver-tagged base-debian image actually in GHCR, regardless of how many older tags exist.
    - Add a unit/integration test that constructs a paginated mock response and asserts the resolver follows the next-link.
    - Document the behavior in `update.rs` so the rationale survives.

    ## Discovered while

    Validating v0.26.2 Phase 2 — the release shipped cleanly (GHCR images live, verifier passed), but `aibox apply` immediately after still reported the pre-v0.26 image. Filed BACK-* as a follow-up for v0.26.3.
---
