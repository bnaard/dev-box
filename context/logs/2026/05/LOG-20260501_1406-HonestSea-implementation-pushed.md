---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260501_1406-HonestSea-implementation-pushed
  created: '2026-05-01T14:06:14+00:00'
spec:
  event_type: implementation.pushed
  timestamp: '2026-05-01T14:06:14+00:00'
  summary: Pushed image provenance doctor checks and preview projection contract coverage.
  actor: codex
  subject: 2598cb5
  subject_kind: commit
  details:
    repository: projectious-work/aibox
    branch: main
    commit: 2598cb5
    scope:
    - Added warning-only doctor checks for image provenance policy drift, mutable vlatest Dockerfile tags, missing Docker label, and missing /etc/aibox-version marker.
    - Added unit tests for image provenance warning detection.
    - Added integration JSON contract tests for addon catalog, workspace manifest, provider backends, and image provenance policy projections.
    - Documented preview projection schemas in the CLI reference.
    validation:
    - cargo fmt --manifest-path cli/Cargo.toml -- --check
    - git diff --check
    - 'cargo test --manifest-path cli/Cargo.toml -j1: 691 unit tests, 55 E2E tier-1 tests, 20 integration tests passed'
    - cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
    - cargo build --manifest-path cli/Cargo.toml --bin aibox
    - 'AIBOX_ADDONS_DIR=/workspace/addons cli/target/debug/aibox doctor: exit 0, 6776 warnings, 0 errors'
    doctor_new_warning: image-provenance-mutable-version is expected in this repo because aibox.toml uses [aibox].version = "latest".
---
