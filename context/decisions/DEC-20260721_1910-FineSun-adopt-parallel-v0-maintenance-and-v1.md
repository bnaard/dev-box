---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260721_1910-FineSun-adopt-parallel-v0-maintenance-and-v1
  created: '2026-07-21T19:10:09+00:00'
spec:
  title: Adopt parallel v0 maintenance and v1 prerelease release branches
  state: accepted
  decision: 'Adopt the version-line branching model from GitHub issue #83. Maintain
    the active v0 release line in parallel with v1 development: use a v0.x development
    branch and designated v0.x release branch for stable v0 releases and hotfixes;
    use a v1.x development branch and v1.x prerelease integration branch for alpha/beta/RC
    releases; create a v1.x stable release branch for GA. Merge tagged release branches
    into main so main remains the complete published history, but tag only on the
    appropriate release or prerelease integration branch.'
  context: aibox will begin a new major evolutionary version soon while continuing
    to support and release the established v0.x line. The existing main-only policy
    cannot express independent development, prerelease, and stable-release authority
    for those concurrent lines.
  rationale: Separate version-line branches make release authority auditable, permit
    v0 maintenance without mixing it with v1 work, and make prerelease promotion explicit.
    This is now justified by actual parallel support needs rather than speculative
    future branching.
  alternatives:
  - option: Continue main-only development and releases
    reason_rejected: Would mix v0 maintenance with v1 prerelease work and make independent
      release authority unclear.
  - option: Freeze v0 while developing v1
    reason_rejected: Conflicts with the requirement to continue supporting and releasing
      v0 in parallel.
  consequences: Release documentation, automation, and branch protection must be updated
    before the first v1 prerelease. Existing v0 release behavior must be migrated
    safely, and main remains a published-history integration branch rather than the
    tagging authority.
  related_workitems:
  - BACK-20260721_1910-HardyClover-parallel-v0-v1-release-branches
  decided_at: '2026-07-21T19:10:09+00:00'
---
