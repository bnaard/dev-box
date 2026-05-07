---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_1042-GentleQuail-release-published
  created: '2026-05-07T10:42:23+00:00'
spec:
  event_type: release.published
  timestamp: '2026-05-07T10:42:23+00:00'
  summary: aibox v0.24.1 release completed including host-side Phase 2
  actor: codex
  subject: v0.24.1
  subject_kind: release
  details:
    version: 0.24.1
    github_release: https://github.com/projectious-work/aibox/releases/tag/v0.24.1
    host_phase_2: completed by user
    macos_binaries: uploaded
    linux_binaries: uploaded
    container_images: pushed to GHCR
    runtime_smoke: passed
    runtime_smoke_logs: dist/release-smoke/v0.24.1/
    generated_runtime_commit: 'd378eaa chore: refresh generated runtime for v0.24.1'
    release_tag_commit: '863ea72 chore: bump CLI version to 0.24.1'
    repo_state: main and origin/main at d378eaa; tag v0.24.1 at 863ea72
    notes: Docs deployment had already run during Linux-side maintain.sh release;
      docs-deploy remains available from the dev-container if needed.
---
