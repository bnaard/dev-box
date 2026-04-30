---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260430_1458-FierceSage-release-phase2-completed
  created: '2026-04-30T14:58:14+00:00'
spec:
  event_type: release.phase2_completed
  timestamp: '2026-04-30T14:58:14+00:00'
  summary: aibox v0.22.0 host-side Phase 2 completed by owner; macOS release assets
    verified on GitHub release; GHCR package API verification unavailable from container
    token.
  actor: owner
  subject: v0.22.0
  subject_kind: Release
  details:
    version: 0.22.0
    tag: v0.22.0
    github_release: https://github.com/projectious-work/aibox/releases/tag/v0.22.0
    macos_assets_verified:
    - aibox-v0.22.0-aarch64-apple-darwin.tar.gz
    - aibox-v0.22.0-x86_64-apple-darwin.tar.gz
    linux_assets_present:
    - aibox-v0.22.0-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.22.0-x86_64-unknown-linux-gnu.tar.gz
    owner_reported: Phase 2 on host done.
    ghcr_verification: Attempted gh api /orgs/projectious-work/packages/container/aibox/versions;
      GitHub returned HTTP 403 requiring read:packages scope.
---
