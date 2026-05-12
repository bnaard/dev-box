---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260430_1432-MerryPanda-release-shipped
  created: '2026-04-30T14:32:39+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-30T14:32:39+00:00'
  summary: aibox v0.22.0 container-side release completed; main and tag pushed, GitHub release created with Linux assets, docs deployed; Phase 2 remains host-side release-host 0.22.0.
  actor: codex
  subject: v0.22.0
  subject_kind: Release
  details:
    version: 0.22.0
    tag: v0.22.0
    commit: 61a1193
    github_release: https://github.com/projectious-work/aibox/releases/tag/v0.22.0
    docs: https://projectious-work.github.io/aibox/
    linux_assets:
    - aibox-v0.22.0-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.22.0-x86_64-unknown-linux-gnu.tar.gz
    phase_2_command: ./scripts/maintain.sh release-host 0.22.0
    dependency_preflight:
      up_to_date:
      - processkit v0.24.0
      - Zellij v0.44.1
      - Yazi v26.1.22
      - ripgrep 15.1.0
      - fd v10.4.2
      - bat v0.26.1
      - eza v0.23.4
      - delta 0.19.2
      - zoxide v0.9.9
      newer_available:
      - fzf 0.71.0 -> v0.72.0
      - ouch 0.6.1 -> 0.7.1
      - starship 1.24.2 -> v1.25.0
      - .devcontainer uv 0.7 -> 0.11.8
      node: devcontainer remains on node:22-slim LTS; latest release observed v25.9.0
---
