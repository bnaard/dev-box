---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260419_0808-SwiftTulip-release-shipped
  created: '2026-04-19T08:08:26+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-18T20:00:00Z'
  summary: "aibox v0.18.7 released end-to-end \u2014 processkit v0.18.2 integrated, Linux + macOS binaries on GitHub release, docs deployed to gh-pages, GHCR images pushed."
  actor: claude-opus-4-7
  details:
    tag: v0.18.7
    commit: 5db8a34
    branch: main
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.18.7
    docs_url: https://projectious-work.github.io/aibox/
    phase_1_linux: completed in devcontainer (preflight, cargo fmt, 617/617 tests, cargo audit, aarch64+x86_64 binaries, tag push, GitHub release, docs build, gh-pages deploy)
    phase_2_macos: completed on host (aarch64+x86_64 darwin binaries + GHCR push)
    processkit_version: v0.18.2
    integrated_commits:
    - 'b010680 feat(v0.18.7): integrate processkit v0.18.2'
    - '044832f chore: track carried session artifacts + v0.18.2 template snapshot'
    - "5db8a34 style: cargo fmt \u2014 collapse two-line assertion"
---
