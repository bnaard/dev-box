---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260415_0938-SilentEagle-sync-wrong-version
  created: '2026-04-15T09:38:53+00:00'
  labels:
    area: sync
    type: bug
spec:
  title: aibox sync records wrong from_version, masks real diff on upgrade
  state: resolved
  resolved_at: '2026-04-15T10:30:00Z'
  resolution: Fixed in commit 02174db — sync now captures pre-install lock before
    computing diff
  type: bug
  priority: high
  description: |
    ## Symptom
    Running `aibox sync` after upgrading from processkit v0.14.0 generated migration file `/workspace/context/migrations/pending/MIG-20260415T093853.md` recording `from_version: v0.16.0, to_version: v0.16.0` with 0 changed upstream and 1 conflict (AGENTS.md only). This is incorrect.

    ## Evidence
    - Migration file: `/workspace/context/migrations/pending/MIG-20260415T093853.md`
    - Lock file: `/workspace/aibox.lock` now pins `v0.16.0`
    - Previous state was `v0.14.0`
    - Between v0.14.0 and v0.16.0:
      - v0.15.0 (2026-04-14): shipped team-creator + session-orientation
      - v0.16.0 (2026-04-15): shipped canonical team-composition schema
    - Upstream processkit issues #40–#52 filed as evidence of integration gaps

    ## Hypothesis
    `aibox sync` writes `context/templates/processkit/v0.16.0/` to disk BEFORE computing the 3-way diff. This causes old-template-on-disk to equal new-template-on-disk, resulting in diff showing nearly everything unchanged. The migration frontmatter `from_version` should have been `v0.14.0`, not `v0.16.0`.

    ## Impact
    Canonical actors schema, team-creator, and session-orientation changes were silently skipped during upgrade, creating a broken state undetected by the migration system.

    ## Next Steps
    Recovery plan being designed by senior-architect. See upcoming delta brief from senior-researcher.

    ## Related Upstream
    GitHub issue: projectious-work/processkit#52 (cumulative integration guidance v0.14.0 → v0.16.0)
---
